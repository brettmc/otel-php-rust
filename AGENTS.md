# AGENTS.md

A PHP extension written in Rust: an OpenTelemetry API/SDK implementation that aims to be a
drop-in replacement for the userland `open-telemetry/opentelemetry-php` packages. Built with
[phper](https://github.com/phper-framework/phper), it targets **PHP 7.0 through 8.5** from one
source tree.

Read this before editing. Most of what follows is a failure mode you cannot see in a diff.

## The one thing to internalise

This code runs *inside* the PHP process, entered by the Zend VM across `extern "C"`
boundaries. A Rust `panic` cannot unwind across such a boundary and there is no PHP frame to
throw into, so **a mistake here aborts the process** — taking an fpm or Apache worker with it —
rather than raising a catchable PHP error. Several conventions below exist only for that
reason, and they are not negotiable for the sake of tidier code.

Consequences you will meet:

- Exit **134** (SIGABRT) — an argument-shape bug, usually "index out of bounds".
- Exit **139** (SIGSEGV) — typically a use-after-free of a `ZVal` held past its request.
- In a `.phpt` run, either shows up as a **dead test with no output**, not a failed
  assertion. See [Tests](#tests).

## Layout

The Cargo workspace is `otel/`, one level below the repository root. Shells and tasks start
in `otel/`; `make` targets live in [otel/Makefile](otel/Makefile).

| Path | What it is |
|---|---|
| `otel/src/lib.rs` | The whole module graph plus `#[php_get_module]`. 40-odd lines. |
| `otel/src/class_registry.rs` | Registers every class and interface in MINIT. **Order is load-bearing** — see below. |
| `otel/src/module.rs` | MINIT/MSHUTDOWN, `phpinfo()` rows, and every `otel.*` INI entry. |
| `otel/src/request.rs` | RINIT/RSHUTDOWN: root spans, header extraction, teardown. The largest module. |
| `otel/src/context/storage.rs` | The `context_id` handle table, thread-locals, `GUARD_STACK`. Second largest, most invariant-heavy. |
| `otel/src/auto/` | Auto-instrumentation. `mod.rs` holds the `cfg` gate and the panic `guard`. |
| `otel/src/auto/plugin/` | Per-framework plugins: `laminas`, `zf1`, `psr18`, shared `db` helpers. |
| `otel/src/trace/`, `logs/`, `context/` | The PHP-visible API surface. A `*_interface.rs` beside most classes. |
| `otel/tests/` | `phpt/` (unit), `auto/` (instrumentation), `http/` (cli-server), `otlp/` (live export). |

`otel/.gitignore` ignores `*.php` — a new test helper `.php` file needs `git add -f`.

## Rules that will bite you

### Optional arguments must be trailing

Both `.optional()` and `.with_default_value()` clear `required`. phper sets
`required_num_args` to the length of the required prefix, as `zend_compile_params` does, so an
optional argument declared before a required one is quietly made required again. That shape
no longer aborts, but the declaration then says something the engine does not do.

Older phper releases used a plain count of required arguments instead. A required argument
after an optional one let Zend accept a call that left a required slot empty, and the handler
indexed a slot that did not exist: SIGABRT, uncatchable.

PHP itself rejects the shape at compile time; phper cannot, because declarations are built at
runtime. **Keeping optional arguments trailing is a convention held by hand.**
`otel/tests/phpt/no-call-shape-aborts.phpt` guards the consequence by calling every exposed
method with no arguments and requiring the process to survive.

### Never index `arguments[n]` — reach for it

Use `arguments.get(n)` with an explicit `None` branch. An ordinary call shape often passes
fewer arguments than the signature declares, and indexing a missing slot panics across the FFI
boundary. See `otel/src/trace/propagation/trace_context_propagator.rs:42`.

The same holds in auto-instrumentation hooks, which see the call as made: `$this->prepare()`
passes nothing to `prepare($sql = null)`. phper's `ExecuteData::get_mut_parameter` panics past
`num_args()`, so hooks use `auto::execute_data::get_mut_parameter`, which returns an `Option`.

### `.optional()` is not free

Making an argument optional to be accommodating can open a crash path: it lets
`setAttribute('k')` past Zend's arity check and into a slot the handler indexes. Where upstream
has an argument required, keep it required. See `otel/src/trace/span_builder.rs:187` and
`otel/src/logs/log_record.rs:259`.

For an optional argument with a natural default, prefer `.with_default_value(...)` over
`.optional()`, class-typed arguments included. From 8.0 the default is recorded in the arg
info, and that is what lets a caller skip the argument by name: `inject($carrier, context: $c)`
raises ArgumentCountError if `$setter` has no default PHP can see. Before 8.0 phper-sys
compiles defaults out, so on 7.x the two behave the same.

### Internal argument types are only checked on 7.x

From 8.0 the engine does not check the declared types of an internal function's arguments.
`setParent(new stdClass)` reaches the handler. So every handler must check what it was given,
and treat a declared type as documentation plus reflection, not a guarantee.

PHP 7.x does check them. That is why some calls that work on 8.x are a TypeError on 7.x, such
as passing an `ArrayIterator` where `iterable` is declared on 7.0.

### Return types are declared but never enforced

An internal function's return type is not verified by the engine. A handler returning Rust `()`
under a non-void hint hands userland `NULL` silently. **Declare the type *and* return a real
value.** See `otel/src/trace/tracer_provider.rs:213`.

### New `extern "C"` entry points go inside `guard(...)`

`otel/src/auto/mod.rs` wraps every engine entry point: a panic in a hook should cost the user
that span, not the process, so it is logged and the instrumented function runs uninstrumented.
Plugin code may assume it is already inside a guard. Instrumentation failures must always
degrade to a plain call.

### Request state is thread-local, never a process static

A request belongs to a thread (ZTS builds exist). `otel/src/request.rs` and
`otel/src/context/storage.rs` keep all request state in thread-locals; a process-wide `Mutex`
would both serialise unrelated requests and let one thread read another's stale entry. Where
locks are used, recover from poisoning with `poisoned.into_inner()` rather than panicking.

`GUARD_STACK` must stay a stack — dropping out of order restores the wrong context.

### Clear PHP values before the disabled check in RSHUTDOWN

Classes are registered even when the extension is disabled for a request, so `Context::with()`
can have stored `ZVal`s. Those are PHP memory, freed with the request: holding them in a
thread-local past that point is a **use-after-free on the next request on a reused worker**,
not a leak. See `otel/src/request.rs:113`.

### Identity is a handle, keys are names

Object identity is always `ZObj::handle()` (mirroring `spl_object_id`), never an address.
The function-observer cache is keyed on the fully-qualified name, not the `zend_function *`:
a closure or `eval`'d function is freed mid-request and its address reused by something
unrelated, which would return the wrong hooks. Providers are keyed on `(pid, …)` because an
exporter owns connections and threads that do not survive a fork.

### Registration order in `class_registry.rs`

Three rules, documented in full at the top of that file:

1. An interface must exist before the class that `implements` it.
2. A class must be registered before anything taking its `StateClass` handle.
3. `Scope`, `Context` and `Storage` refer to each other, so they cannot follow rule 2 — they
   are built in two phases (`new_*` then `build_*`) instead of a single `make_*`.

Rule 1 also applies to any type merely *named in a signature*, not just one that is
`implements`ed. `Globals` is the userland entry point and registers last. Within a group,
order is free.

### 32-bit builds

Constants that depend on integer width are computed, not hardcoded — e.g.
`1i64 << (i64::BITS - 1)` in `storage.rs`. Hardcoding the 64-bit value silently produces the
wrong flags on a 32-bit build.

## PHP version portability

`otel/build.rs` sets exactly one cfg: `otel_observer_supported` (PHP 8+) or
`otel_observer_not_supported` (7.x). `otel/src/auto/mod.rs` compiles exactly one backend —
`observer.rs` (Zend observer API) or `execute.rs` (`zend_execute_ex` swap).

**Only one is ever built, so a normal build does not type-check the other.** To check the 7.x
file from an 8.x container:

```shell
RUSTFLAGS="--cfg otel_observer_not_supported" cargo check --features test
```

This is why CI runs clippy on all 11 PHP versions rather than one — a single-version run is
blind to one of those files entirely.

For Zend structs whose field types differ across versions, **infer the type rather than
annotate it**. `auto_globals_jit` is `zend_bool` (`u8`) on 7.x and a real `bool` on 8+, so
neither annotation compiles everywhere and a cast is a no-op clippy denies on whichever
version already matches. Comparing against `Default::default()` is correct and lint-clean
everywhere — see `otel/src/request.rs:518`.

Other version facts:

- PHP 7.x cannot write to stdout/stderr during MSHUTDOWN, so shutdown-log and
  console-exporter assertions must skip or redirect to a file on 7.x.
- `phper-sys` compiles class-valued type hints (`ZEND_ARG_OBJ_INFO`) and argument default
  values out before PHP 8.0, so tests reflecting over those signatures have nothing to read
  on 7.x.
- Internal functions cannot be instrumented before PHP 8.2.
- A test that *can* run on 7.x should guard the version-specific assertion rather than skip
  the whole file, as `interface-surface.phpt` does.

## What phper cannot express

As of phper 0.17.5 / `phper-sys` 0.15.6 at git rev `1d7fbf2`, pinned in `otel/Cargo.toml`:

- No intersection types. `context.rs` and `tracer_provider.rs` declare the half that is true.
- No `Callable` property type.
- No promoted constructor properties. `ContextKey` declares a readonly property and writes it
  from the constructor instead.

Available, with the PHP version each needs:

- Argument unions (`ArgumentTypeHint::Union`) and `false`, 8.0+. Before 8.0 a union is
  declared untyped.
- Defaults on class-typed arguments, 8.0+.
- `ClassEntity::set_final()`, every version.
- `add_readonly_property`, readonly on 8.1+, typed on 8.0, untyped on 7.x.
- `abstract`, via `add_abstract_method`.

A final internal class cannot be built with `ReflectionClass::newInstanceWithoutConstructor()`.
Tests that need a constructor-less instance use `unserialize('O:<len>:"<class>":0:{}')`, as
`no-call-shape-aborts.phpt` does.

Where a workaround changed a signature, the reason is commented at the signature itself. Keep
that up, and prefer widening a type over narrowing one.

Interfaces here are load-bearing rather than decorative: the engine rejects an implementation
that omits a method, which is what stops `Span` and `NonRecordingSpan` drifting from each
other or from upstream. `NonRecordingSpan`'s methods are all no-ops but their signatures must
match `Span`'s exactly — differing arity or types make the same call legal against one and a
`TypeError` against the other.

## Build and test

```shell
make build-test     # debug build + phpize (for run-tests.php) -> modules/otel.so
make test           # cargo test --features test, then every phpt suite
make clippy         # cargo clippy --all-targets --features test
make fmt            # cargo fmt; CI fails on anything `make fmt-check` reports
make test-phpt      # unit tests only
make test-auto      # auto-instrumentation (needs composer fixtures)
make test-export    # otlp, needs the collector; not run in CI
make test-http      # cli-server root spans
make test-all       # make update, then everything
make re / make ri   # reflection / phpinfo dump of the built extension
```

`--features test` compiles `auto/plugin/test.rs`, which provides the `TestPlugin` the phpt
assertions rely on. Clippy must be run with it too, or that file is not linted.

`php-config --extension-dir`/`otel.so` is symlinked to `otel/modules/otel.so`, so
`php -d extension=otel` picks up whatever you last built with no install step.

Switching PHP versions by hand needs `make clean`: `php_bindings.rs` is generated by phper and
is version-specific. The dev containers give each version its own `target/` volume, so
switching between *them* needs no clean.

### Tests

`.phpt` is php-src's own format: `--TEST--`, `--SKIPIF--`, `--EXTENSIONS--`, `--INI--`,
`--ENV--`, `--FILE--`, `--EXPECT--`/`--EXPECTF--`. Tests assert by reading back the in-process
memory exporter, or by matching console-exporter output.

**A crash is a dead test, not a failed assertion** — no output at all, which an
expectation of "no errors" would pass. So every test must **assert on concrete values and end
with a sentinel line** (`DONE`); reaching the sentinel is itself the thing being proved. The
existing tests say so in their own comments: `declared-defaults-readable.phpt:29`,
`userland-interface-implementations.phpt:20`, `no-call-shape-aborts.phpt:12`.

Any fix should land with phpt coverage pinning the **upstream-observable behaviour**, not
merely that the call does not error.

The composer fixtures (`tests/auto/{laminas,zf1,psr18}`) need `make update`, which installs
only what the running PHP can resolve (zf1 always, laminas 7.1+, psr18 7.2+). `make clean`
removes them. In the dev containers each version has its own `vendor/` volume, populated by a
`postCreateCommand`.

`tests/otlp/` needs a collector at `collector:4317`/`:4318` and is **not run in CI**, so run
it locally when touching export.

The sqlite fixtures (`tests/auto/{laminas,zf1}/data/test.sqlite`) are **committed binaries**,
so a test that writes to one leaves the working tree dirty for everyone. Keep db tests
read-only, or wrap writes in a transaction and roll back — a rolled-back transaction leaves
the file byte-identical. Statement-level DDL like `vacuum` or `analyze` rewrites the file even
inside a transaction, and `vacuum` is not what makes a span name "OTHER" anyway: the regexes
in `auto/utils.rs` only recognise SELECT/INSERT/UPDATE/DELETE, so any unrecognised read-only
statement does it.

### Reading CI

The test matrix sets job-level `continue-on-error: true`, so **a red matrix entry does not
fail the workflow**. Read the matrix itself rather than trusting the green check. The
`latest-deps` job is `continue-on-error` on purpose too.

`otel/Cargo.lock` is committed — a `cdylib` with `publish = false`, so no downstream consumer
sees it, and it keeps the 11-version matrix and every contributor on one dependency graph.

## Conventions

- Match the comment density of the surrounding code. This codebase explains **why**, not what,
  and especially why an obvious-looking simplification is wrong. That is load-bearing here:
  most of the traps above are invisible at the call site. If you work one out, write it down
  next to the code.
- Run `make fmt` before committing. CI runs `cargo fmt --check` in its own job, outside the
  `continue-on-error` matrix, so unformatted code fails the workflow.
- Known blind spot: nothing compares the **whole** API surface to upstream.
  `interface-surface.phpt` pins 13 interfaces by hand, which is a net, not a snapshot.
