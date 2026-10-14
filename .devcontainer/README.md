# Dev containers

One dev container per supported PHP version, each with the Rust toolchain and
PHP development headers.

| Dev container            | PHP   |
|--------------------------|-------|
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

The workspace opens at `/workspace/otel`, so the `make` targets in
`otel/Makefile` work directly:

```shell
make build-test   # build with debug symbols
make test         # cargo test + phpt tests
make test-export  # otlp tests, against the bundled collector
make test-auto    # auto-instrumentation tests
make test-http    # cli-server tests
make test-all     # composer fixtures, then every suite
```

The `tests/auto` suite needs its composer fixtures installed, so run
`make update` (or `make test-all`, which does it for you) before `make test-auto`
in a fresh container — otherwise those tests fail on missing `vendor/` dirs.

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
are installed. See
[AI assistants and other personal tooling](#ai-assistants-and-other-personal-tooling)
to add your own.

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

[../otel/.vscode/tasks.json](../otel/.vscode/tasks.json) is committed, so the
tasks are available as soon as the dev container opens — no copy step. It lives
under `otel/` because that is the `workspaceFolder`, which is where VS Code
looks for `.vscode/tasks.json`.

`.gitignore` ignores `**/.vscode/*` but un-ignores `tasks.json`, so these shared
tasks are versioned while personal `settings.json` / `launch.json` stay out of
the repository.

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
and the tool-config volume are shared across all of them.

Each version also uses its own compose project name, so several dev
containers can run side by side.

To reclaim the disk a version's build cache is using:

```shell
docker volume rm otel-php-rust-target-php84
```

## Git

The dev containers set `"git.openRepositoryInParentFolders": "always"`, because
the workspace opens at `/workspace/otel` while the repository root is one level
up at `/workspace`. By default VS Code only offers to open a repository found in
a *parent* of the folder you opened, behind a prompt — so without this setting
the Source Control view starts empty and you get no diffs or gutter
annotations. The setting is scoped to these dev containers only.

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

Nothing of this kind is installed, and the committed configuration does not
assume you use any of it. If you want a CLI assistant, a language server or any
other personal tool inside the dev containers, add it through your own VS Code
**user** settings — nothing to commit, and no effect on anyone else.

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
  `/home/php-rust/.config/dev-tools`, shared across all seven versions and
  inert if nothing writes to it. Point your tool at it with whatever variable
  it uses for its config directory, set in `.env`.

  If a tool keeps some state *outside* its config directory, check whether it
  has a variable to relocate that too — otherwise the volume alone will not be
  enough to survive a rebuild.

If a tool wraps a hosted API and you repoint it at an internal gateway, expect
to set a base URL, a token, and possibly resource or model names that the
gateway publishes rather than the vendor's own. Those names are often absent
from the tool's built-in list, so warnings about an unrecognised model or
capability are common and frequently cosmetic — check whether the tool still
works, and whether the message is only on stderr, before chasing it.

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

These are additive: the repository's top-level `Dockerfile`, `docker-compose.yaml`
and `make bash` workflow are unchanged and still work. The dev containers use
their own `Dockerfile` here, based on `debian:bookworm`, whose
[sury](https://packages.sury.org/php/) repository carries every PHP version
from 7.4 to 8.5 — the top-level image is `debian:bullseye` and is not changed
by any of this.

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
