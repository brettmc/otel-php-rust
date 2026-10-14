#!/usr/bin/env bash
#
# Builds the dev container image for one PHP version.
#
# Each devcontainer.json runs this as its initializeCommand, which the Dev
# Containers extension runs on the host before starting the container. The
# compose services therefore name a prebuilt `image:` and carry no `build:`
# key of their own.
#
# That split is deliberate. When a compose service has a `build:` key, the
# extension rewrites it to point at a `Dockerfile-with-features` it generates
# in a temporary directory -- it does this even with no dev container
# `features`, just to stamp its metadata label onto the image. Docker Compose
# v5 builds through bake, and bake refuses to read a Dockerfile from outside
# the compose project directory, so that generated path fails with:
#
#   additional privileges requested: pass "--allow=fs.read=/.../
#   Dockerfile-with-features" to grant requested privileges
#
# A service with `image:` and no `build:` is never rewritten this way, so
# building here sidesteps the problem rather than granting bake extra
# privileges. See .devcontainer/README.md.

set -euo pipefail

php_version="${1:?usage: build-image.sh <php-version>}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "building otel-php-rust-dev:${php_version} (dev container image)..."

# Layers are cached, so this is a no-op after the first run for a version.
docker build \
  --build-arg "PHP_VERSION=${php_version}" \
  --tag "otel-php-rust-dev:${php_version}" \
  "${here}"
