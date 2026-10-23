# Dev containers

One dev container per supported PHP version, each with the Rust toolchain and
PHP development headers.

| Dev container            | PHP   |
|--------------------------|-------|
| `otel-php-rust (PHP 7.0)`| 7.0   |
| `otel-php-rust (PHP 7.1)`| 7.1   |
| `otel-php-rust (PHP 7.2)`| 7.2   |
| `otel-php-rust (PHP 7.3)`| 7.3   |
| `otel-php-rust (PHP 7.4)`| 7.4   |
| `otel-php-rust (PHP 8.0)`| 8.0   |
| `otel-php-rust (PHP 8.1)`| 8.1   |
| `otel-php-rust (PHP 8.2)`| 8.2   |
| `otel-php-rust (PHP 8.3)`| 8.3   |
| `otel-php-rust (PHP 8.4)`| 8.4   |
| `otel-php-rust (PHP 8.5)`| 8.5   |

## Opening one

In VS Code: **Dev Containers: Reopen in Container**, then pick the PHP version
you want. To switch versions, run the command again and pick another — see
[Switching versions](#switching-versions).

The workspace opens at `/workspace`, the repository root, so the committed
`.vscode/`, `.github/`, the top-level `Makefile` and `CONTRIBUTING.md` are all
visible and editable, and git sees its repository at the workspace root rather
than above it.

Terminals and tasks still start in `/workspace/otel`, so the `make` targets in
`otel/Makefile` work directly:

```shell
make build-test   # build with debug symbols
make test         # cargo test + phpt tests
make test-export  # otlp tests, against the bundled collector
make test-auto    # auto-instrumentation tests
make test-http    # cli-server tests
make test-all     # composer fixtures, then every suite
```

The `tests/auto` suite needs its composer fixtures installed. That happens for you when the
container is created — see [Composer fixtures](#composer-fixtures) — so `make test-auto` works
directly. If you ever need to redo it, `make update` installs them and `make test-all` runs it
for you.

## How the image is built

Each `devcontainer.json` runs [build-image.sh](build-image.sh) as its
`initializeCommand`, which the extension runs on the host before starting the
container. The compose services name a prebuilt `image:` and have **no
`build:` key**.

That is not incidental — a `build:` key makes the extension generate a
Dockerfile in a temporary directory, which Docker Compose v5 then refuses to
build (see [Troubleshooting](#additional-privileges-requested-pass---allowfsread)).
Please do not add one back.

Opening a dev container still builds the image for you, so there is no manual
step. Layers are cached, so it is effectively instant once a version has been
built. To build one by hand:

```shell
.devcontainer/build-image.sh 8.4
```

## What's in the container

* Rust stable (`rustup`, `cargo`, `clippy`, `rustfmt`, `rust-src`)
* `php<version>`-cli, -dev, -curl, -sqlite3, plus `-dbgsym` debug symbols
* `gdb`, `valgrind`, `strace` for the low-level debugging described in
  [CONTRIBUTING.md](../CONTRIBUTING.md#low-level-debugging)
* `composer`, for the `tests/auto/*` fixture dependencies
* `git` and `openssh-client`, so commits can be signed and pushed over SSH
  from inside the container (see [Git](#git))
* VS Code extensions for Rust (`rust-analyzer`, `CodeLLDB`), TOML, Cargo
  dependencies, and PHP

That is the whole toolchain: no AI assistants, linters or other personal tools
are installed — including Claude Code, though the committed configuration does
make its state persist if you install it yourself. See
[AI assistants and other personal tooling](#ai-assistants-and-other-personal-tooling)
to add your own, and [Claude Code](#claude-code) for that one in particular.

`php-config --extension-dir`/`otel.so` is symlinked to
`/workspace/otel/modules/otel.so`, so `php -d extension=otel` picks up
whatever you last built, with no install step.

## Your own environment variables

The dev containers read the repository's `.env`, which is not committed, so
anything you put there is passed into the container — a place to configure
tools of your own that expect credentials or settings in the environment.

It is wired up as an optional `env_file` in
[compose.base.yaml](compose.base.yaml), so the container still starts when
`.env` is absent:

```yaml
env_file:
  - path: ../.env
    required: false
```

Variables set under `environment:` in that file take precedence, so `.env`
cannot change `CARGO_HOME` or `TEST_PHP_EXECUTABLE`.

Keep your own variables out of `.env.dist` — it is committed, and is the
template people copy to `.env`.

## Tasks

[../.vscode/tasks.json](../.vscode/tasks.json) is committed, so the tasks are
available as soon as the dev container opens — no copy step. It sits at the
repository root because that is the `workspaceFolder`, which is where VS Code
looks for `.vscode/tasks.json`. Every task sets `cwd` to
`${workspaceFolder}/otel`, since each one runs a target from `otel/Makefile`.

`.gitignore` ignores `**/.vscode/*` but un-ignores `tasks.json` and
`settings.json`, so the shared tasks and rust-analyzer configuration are
versioned while personal `launch.json` and the rest stay out of the repository.

Run them with **Tasks: Run Task**, or **Run Test Task** / **Run Build Task**
for the defaults (`make test` and `make build-test`).

The test tasks attach a problem matcher that turns `run-tests.php` failures
into entries in the Problems panel, each linked to the failing `.phpt` file.
`NO_COLOR=1` is set for every task because the matcher cannot read the
`FAIL`/`PASS` lines through the ANSI colour codes.

Two things about that matcher are load-bearing, both because `run-tests.php`
reports a verdict per *file* and never a line number:

* `"kind": "file"` is required. VS Code rejects a pattern outright — *"The
  problem pattern is invalid. It must either have kind: `file` or have a line
  or location match group"* — if it has neither.
* The regex matches on `\b`, not `^`. `run-tests.php` writes
  `TEST n/m [file]\r` and then the verdict, so `FAIL` sits mid-line on the
  same physical line and an anchored pattern silently never matches.

`test: current phpt file` runs only the file open in the editor, and
`test: JUnit report` writes `otel/junit.xml` — see
[Test Explorer](#test-explorer) below.

## Editor support for `.phpt`

Two gaps worth knowing about, both because `.phpt` is php-src's own test
format and has almost no tooling outside php-src itself.

### Syntax highlighting

No extension provides a `.phpt` grammar — not on the VS Code Marketplace, not
on Open VSX, and VS Code's built-in PHP grammar claims only `.php`, `.php4`,
`.php5`, `.phtml` and `.ctp`.

As a partial measure, the dev containers map `*.phpt` to the PHP language.
That highlights the `--FILE--` section, which is the bulk of most tests, but
the `--TEST--`, `--INI--`, `--ENV--` and `--EXPECT--` headers are not PHP and
will look like stray text. `intelephense.files.exclude` keeps the language
server from reporting phantom errors in them.

Proper highlighting would need a small TextMate grammar that treats the
sections as a container and embeds `source.php` inside `--FILE--`.

### Test Explorer

No extension runs `.phpt` either; every PHP test explorer on the marketplace
is PHPUnit/Pest-only. The tasks above are the practical substitute.

If you do want a Test Explorer tree, `run-tests.php` can emit JUnit XML, which
is a standard format a custom extension could consume:

```shell
TEST_PHP_JUNIT=junit.xml php run-tests.php -q tests/phpt/
```

Failures carry the expected/actual diff in the `<failure>` body, and suites are
nested per directory.

## The collector

Each dev container also starts an `otel/opentelemetry-collector-contrib`
service reachable as the hostname `collector`. The tests in `otel/tests/otlp`
hardcode `http://collector:4317` and `:4318`, so `make test-export` works
without any extra setup. The collector uses
[files/collector/otel-collector-config.yml](../files/collector/otel-collector-config.yml)
and logs received telemetry at `debug` verbosity:

```shell
docker compose -p otel-php-rust-php84 logs -f collector
```

## Switching versions

`phper` generates PHP-version-specific bindings under `target/`, which is why
[CONTRIBUTING.md](../CONTRIBUTING.md#switching-php-versions) says to
`make clean` when switching versions by hand.

Because that volume is mounted *at* `otel/target`, `cargo clean` cannot remove
the directory itself ("Device or resource busy"). `make clean` detects the
mount point and empties it in place instead; outside a dev container it still
runs `cargo clean` as before.

The dev containers give each PHP version its own `target/` volume
(`otel-php-rust-target-php84` and friends), so **you do not need to
`make clean` when switching between them**, and each version keeps a warm
build cache. The cargo download caches (`~/.cargo/registry`, `~/.cargo/git`)
and the two personal-tooling volumes (`dev-tools`, `claude-config`) are shared
across all of them. The composer fixtures are per-version, like `target/` —
see [Composer fixtures](#composer-fixtures).

Each version also uses its own compose project name, so several dev
containers can run side by side.

To reclaim the disk a version's build cache is using:

```shell
docker volume rm otel-php-rust-target-php84
```

## Composer fixtures

The `tests/auto/{laminas,zf1,psr18}` fixtures get a volume each, per version, for the same
reason `target/` does: composer resolves dependencies against the PHP it runs under, so a
`vendor/` installed on 8.x holds code that does not parse on 7.x.

They used to sit on the bind-mounted repository, which made them shared — whichever version
installed them last was what every container saw, and the `auto/laminas` tests then failed
with parse errors *inside* `vendor/` rather than anything to do with the extension. That is
now impossible between dev containers; it still applies on the host, where the working tree
is all there is.

A fresh volume is empty and the fixtures' `composer.lock` is gitignored, so each
`devcontainer.json` populates it with a `postCreateCommand`:

```jsonc
"postCreateCommand": "make -C ${containerWorkspaceFolder}/otel update"
```

The first open of a version therefore spends a minute or so on composer. After that the
volume is warm and reopening is immediate. `postCreate` rather than `postStart` because it
only needs to happen once per container.

`make update` installs only the fixtures the running PHP can resolve — `zf1` always,
`laminas` on 7.1+, `psr18` on 7.2+ — mirroring the gates in `.github/workflows/test.yaml`
rather than duplicating the version matrix. On 8.5 it adds `--ignore-platform-req=php` for
laminas, for the reason given in that workflow. Running all three unconditionally is what
used to fail on 7.0 and 7.1.

To force a reinstall, remove the volume and rebuild, or just run `make update` again:

```shell
docker volume rm otel-php-rust-laminas-vendor-php84
```

One wrinkle worth knowing if you add a volume of your own: Docker seeds a new named volume
from the image directory it covers, and a volume mounted at a path the image does **not**
have is created root-owned and unwritable by `php-rust`. Composer then fails with a bare
`Permission denied`. That is why [Dockerfile](Dockerfile) pre-creates and `chown`s each of
these mount points.

## Git

The workspace opens at the repository root, so the Source Control view works
with no extra configuration. (It previously opened at `/workspace/otel`, one
level *below* the repository root, which needed
`"git.openRepositoryInParentFolders": "always"` or Source Control started
empty — that setting is gone.)

VS Code forwards your SSH agent into the container, so pushing to a
`git@github.com:` remote works without copying any private key in. The first
push asks you to confirm GitHub's host key, because the container starts with
an empty `known_hosts`.

### Signed commits

VS Code copies your `~/.gitconfig` into the container but never copies key
*files*. If you sign commits with an SSH key (`gpg.format=ssh`), your config
therefore arrives pointing `user.signingkey` at a path that does not exist
here, and every commit fails:

```
error: Couldn't load public key /home/php-rust/.ssh/id_ed25519: No such file or directory
fatal: failed to write commit object
```

git needs the *public* half on disk to build the signature, even though the
private half stays in the agent on your host and is never written into the
container. [setup-signing-key.sh](setup-signing-key.sh) runs on container
start and writes just that file, taking the path from your own
`user.signingkey` and the key from the forwarded agent.

It does nothing at all if you do not sign commits, or if `user.signingkey` is
a `key::ssh-ed25519 AAAA...` literal, since git hands that form straight to
the agent. It never overwrites a key that is already there.

If your agent holds several keys it picks the one whose comment matches your
`user.email`. When nothing matches it writes *no* key rather than guess —
signing with the wrong key still produces a commit, just one GitHub reports as
unverified, which is an obscure thing to trace back. In that case write it
yourself and restart the container:

```shell
mkdir -p ~/.ssh && chmod 700 ~/.ssh
ssh-add -L | grep 'your-key-comment' > ~/.ssh/id_ed25519.pub
```

The file lives in the container's own filesystem, so it is rewritten after
every rebuild. For GitHub to show commits as verified, the key must also be
registered on your account as a **signing** key — an authentication key alone
is not enough.

`git verify-commit` in the container reports `N` (unsigned) even for a
correctly signed commit unless you also set `gpg.ssh.allowedSignersFile`;
that only affects local verification, not the signature itself.

## AI assistants and other personal tooling

Nothing of this kind is installed. If you want a CLI assistant, a language
server or any other personal tool inside the dev containers, add it through
your own VS Code **user** settings — nothing to commit, and no effect on anyone
else.

The committed configuration does not *require* any of it. The one exception is
that it provides the volume and environment for Claude Code's state, since that
is the awkward part of using it across eleven containers — see
[Claude Code](#claude-code). It is inert if you do not install it.

Dev container *features* install software into the image:

```jsonc
"dev.containers.defaultFeatures": {
  "ghcr.io/devcontainers/features/node:1": {}
}
```

and `dev.containers.defaultExtensions` installs VS Code extensions:

```jsonc
"dev.containers.defaultExtensions": [
  "your.extension-id"
]
```

Both apply to every dev container you open, this project's included. They are
easy to mix up: a feature installs the program, an extension only installs
editor integration. A tool you want to run in the terminal needs
`defaultFeatures` — which usually pulls its extension in as well.

Features are baked into the image, so after changing either setting you must
**rebuild** rather than reopen (**Dev Containers: Rebuild Container**). An
existing container is reused as-is, and the new tool will appear to be missing.

There is deliberately no `devcontainer.local.json`: the Dev Containers spec has
no local-override merge mechanism, so user settings are the supported way to do
this.

### Credentials and settings for those tools

Your host dotfiles are **not** copied into the container; only `~/.gitconfig`
and the SSH agent are forwarded (see [Git](#git)). So a tool that reads a
config file or an API token on the host needs one of two mechanisms here:

* **The repository's `.env`**, for anything configured by environment variable
  — see [Your own environment variables](#your-own-environment-variables). A
  tool that authenticates with a token, or points at an internal proxy, is
  usually configured entirely this way, in which case it works as soon as the
  container starts.
* **A named volume**, for a tool that writes a credential or state file and
  would otherwise lose it on every rebuild. `compose.base.yaml` carries one
  general-purpose volume for this, mounted at
  `/home/php-rust/.config/dev-tools`, shared across all eleven versions and
  inert if nothing writes to it. Point your tool at it with whatever variable
  it uses for its config directory, set in `.env`.

  If a tool keeps some state *outside* its config directory, check whether it
  has a variable to relocate that too — otherwise the volume alone will not be
  enough to survive a rebuild. That is common enough to be worth checking
  first: Claude Code needs exactly this, and the next section shows what it
  took.

If a tool wraps a hosted API and you repoint it at an internal gateway, expect
to set a base URL, a token, and possibly resource or model names that the
gateway publishes rather than the vendor's own. Those names are often absent
from the tool's built-in list, so warnings about an unrecognised model or
capability are common and frequently cosmetic — check whether the tool still
works, and whether the message is only on stderr, before chasing it.

### Claude Code

Nothing here installs it, and the configuration below is inert if you do not use it. But
because it keeps state in a known place, `compose.base.yaml` provides the volume and the two
environment variables that make it behave the same in all eleven containers — which is
otherwise the awkward part of running it across this many.

To install it, in your own VS Code **user** settings:

```jsonc
"dev.containers.defaultFeatures": {
  "ghcr.io/devcontainers/features/node:1": {},
  "ghcr.io/anthropics/devcontainer-features/claude-code:1.0": {}
},
"dev.containers.defaultExtensions": [
  "anthropic.claude-code"
]
```

The feature installs the CLI; the extension adds the editor panel. Both run *inside* the
container and share one `~/.claude`. The `node` feature is there because the Claude Code
feature needs Node and this image does not carry it — without it the build stops with
`Failed to install Node.js and npm`. Features are baked into the image, so **rebuild** rather
than reopen after adding them.

The `:1.0` tag pins the feature's install script, not the CLI, which installs latest and
auto-updates. `DISABLE_AUTOUPDATER=1` in your `.env` stops that.

What the committed configuration does for you:

```yaml
volumes:
  - claude-config:/home/php-rust/.claude
environment:
  CLAUDE_CONFIG_DIR: /home/php-rust/.claude
  CLAUDE_CODE_PROJECT_DIR_NAME: otel-php-rust
```

* **One volume, `otel-php-rust-claude-config`, shared by every version.** Without it the
  container's home directory is discarded on rebuild and you sign in again each time — eleven
  containers, each rebuilt whenever a feature changes, is a lot of signing in.
* **`CLAUDE_CONFIG_DIR`** is the part that is easy to miss. The auth token, settings and
  session history live under `~/.claude`, but the OAuth account and per-project trust live in
  `~/.claude.json` — a separate file *outside* that directory, so the volume alone does not
  keep you signed in. This variable moves that file into the volume too.
* **`CLAUDE_CODE_PROJECT_DIR_NAME`** fixes the project identity used for memory (below).

Credentials come from the repository's `.env` like any other tool's
(`ANTHROPIC_AUTH_TOKEN`, `ANTHROPIC_BASE_URL`, and whatever model names your gateway
publishes) — see
[Your own environment variables](#your-own-environment-variables).

#### Shared knowledge across sessions

Two mechanisms, and they answer different needs.

**Instructions you write** go in `AGENTS.md` at the repository root, which is committed and
read automatically. It holds what the codebase cannot show on its own — the traps where the
obvious change aborts a PHP worker. Claude reads a `CLAUDE.md` *instead* of `AGENTS.md` if one
exists in or above the working directory, so there deliberately isn't one here.

For personal instructions you do not want committed, either add a `CLAUDE.local.md` or import
one from your home directory. Prefer the import: `.gitignore` already covers
`CLAUDE.local.md`, but adding one **stops Claude reading `AGENTS.md`**, which is probably not
what you want. An import from outside the workspace needs one-time approval, and a file in
your home directory is on the container's own filesystem, so point it at the volume:

```markdown
@~/.claude/otel-php-rust-notes.md
```

**Notes Claude keeps itself** — auto memory — live in
`<config dir>/projects/<project>/memory/`, as a `MEMORY.md` index plus a file per fact. Since
that is inside the volume, it persists across rebuilds and is the *same* memory in all eleven
containers: something learned on 8.4 is there on 7.0.

`CLAUDE_CODE_PROJECT_DIR_NAME` is what makes that reliable. `<project>` is normally derived
from the repository path, which here is `/workspace` in every container — so they would agree
anyway, but so would any *other* project opened at `/workspace` in a container sharing this
volume, which is the real hazard. Naming it explicitly keeps this project's memory to this
project.

Auto memory is machine-local and is **not** shared with the Claude Code you run on the host:
same repository, different machine as far as it is concerned. The memory directory is also
exempt from the `cleanupPeriodDays` sweep that deletes old transcripts.

#### Sharing host config and memory

Every `devcontainer.json` also loads `.devcontainer/compose.local.yml`, which is not
committed. `build-image.sh` creates it as an empty `services: {}` when it is missing. Use it
to layer your host's Claude Code files over the volume:

```yaml
services:
  php:
    volumes:
      - ~/.claude/settings.json:/home/php-rust/.claude/settings.json:ro
      - ~/.claude/CLAUDE.md:/home/php-rust/.claude/CLAUDE.md:ro
      - ~/.claude/projects/<host-project>/memory:/home/php-rust/.claude/projects/otel-php-rust/memory
```

`<host-project>` is the repository's host path with each `/` and `.` replaced by `-`, as
listed under `~/.claude/projects`. Add any file your `CLAUDE.md` imports by `~/` path, since
the import resolves inside the container. Only mount paths that exist: Docker creates a
missing host path as an empty directory.

Docker creates the mount point for a nested mount as root. The image creates
`projects/otel-php-rust/memory` as `php-rust` so that a new volume is seeded with the right
owner. A volume created before that needs it made once, as the container user:

```shell
docker run --rm -u 1000:1000 -v otel-php-rust-claude-config:/c alpine \
  mkdir -p /c/projects/otel-php-rust/memory
```

One caveat about the shared volume: it is one directory, so several containers running Claude
Code *at the same time* are writing to it at the same time. Sequential use across versions,
which is the normal way to work here, is fine. If you want per-version isolation instead, move
the `claude-config` volume out of `compose.base.yaml` into each `php*/compose.yaml` with a
per-version `name:`, as `target` is done — at the cost of signing in eleven times and eleven
separate memories.

## Troubleshooting

### `additional privileges requested: pass "--allow=fs.read=..."`

The container fails to build, naming a path under
`.../devcontainercli/container-features/.../Dockerfile-with-features`:

```
additional privileges requested: pass "--allow=fs.read=/.../Dockerfile-with-features" to grant requested privileges
```

**This should not happen any more** — it is why the compose services have no
`build:` key. If you see it, something has reintroduced one.

When a compose service has a `build:` key, the extension rewrites it to point
at a `Dockerfile-with-features` that it generates in a temporary directory,
while keeping *our* `context` — the repository. It does this even with no dev
container `features`, purely to stamp its metadata label onto the image.
Docker Compose v5 builds through bake, and bake refuses to read a Dockerfile
that sits outside the context it was given.

A generated Dockerfile is not in itself the problem: when the extension adds
one for a *feature*, it generates an empty context beside it in the same
temporary tree, and bake allows that. The failure needs the mismatch — a
temporary Dockerfile paired with a context in the repository.

A service with `image:` and no `build:` is never rewritten, so the images are
built on the host by [build-image.sh](build-image.sh) instead — see
[How the image is built](#how-the-image-is-built). Nothing is disabled and no
extra privilege is granted.

Things that do *not* help, in case you are tempted:

* `COMPOSE_BAKE=false` / `=0` — ignored by Compose v5, which still logs
  `load local bake definitions` and still fails.
* `build.entitlements` in [compose.base.yaml](compose.base.yaml) — no effect;
  the extension's generated override is what actually runs.
* `dev.containers.dockerComposePath` — the extension invokes `docker compose`,
  not standalone `docker-compose`, so a wrapper there is never called.
* `"updateRemoteUserUID": false` — the image is extended regardless of UID
  syncing, so this changes nothing.
* Setting `TMPDIR` under the repository does work, because bake trusts
  anything inside the project directory, but only if VS Code itself was
  launched with it set — not when reopening a container. Too fragile to rely
  on.

The blunt workarounds, for reference only: `DOCKER_BUILDKIT=0` forces the
deprecated classic builder, and `BUILDX_BAKE_ENTITLEMENTS_FS=0` disables the
entitlement check for *every* build in that environment.

## Relationship to `make bash`

These are additive: the repository's top-level `Dockerfile`,
`docker-compose.yaml` and `make bash` workflow still work. The dev containers
use their own `Dockerfile` here, based on `debian:bookworm`, whose
[sury](https://packages.sury.org/php/) repository carries every PHP version
from 7.0 to 8.5 — the top-level image is `debian:bullseye` and is not changed
by any of this.

The two now agree on paths: `make bash` also mounts the whole repository at
`/workspace` and starts in `/workspace/otel`, so a path from one works in the
other. It used to mount only `otel/`, at `/usr/src/myapp`. If you have an
existing `php-rust:<version>` image from before that change, rebuild it with
`make build-image` — the `otel.so` symlink is baked into the image and the old
one points at a path that no longer exists.

## PHP 8.5

The dev container builds and PHP 8.5 itself runs, but the extension does not
currently compile against it: the pinned `phper` revision does not yet support
PHP 8.5's internal API (`HASH_KEY_IS_STRING`/`HASH_KEY_IS_LONG` and some
signature changes). The container is included so the build can be re-tried as
`phper` adds support. CI currently covers 7.0 through 8.4.

## PHP 7.0 - 7.3

Not included, because bookworm's sury repository does not carry them. CI still
tests these versions, and the top-level `Dockerfile` (`debian:bullseye`) can
still build them via `make bash`.
