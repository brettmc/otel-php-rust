# Dev container image for otel-php-rust.
#
# Separate from the repository's top-level Dockerfile (which backs `make bash`):
# this one is based on bookworm, whose sury repository carries php7.4 through
# php8.5, so a single base covers every version we ship a dev container for.
FROM composer:lts AS composer
FROM debian:bookworm

ARG PHP_VERSION=8.4
ARG USERNAME=php-rust
ARG USER_UID=1000
ARG USER_GID=1000

ENV DEBIAN_FRONTEND=noninteractive

RUN groupadd -g ${USER_GID} ${USERNAME} \
  && useradd -g ${USER_GID} -u ${USER_UID} --create-home --shell /bin/bash ${USERNAME}

# build toolchain for phper/bindgen, plus the debugging tools CONTRIBUTING.md leans on
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    llvm-dev \
    libclang-dev \
    clang \
    build-essential \
    pkg-config \
    autoconf \
    automake \
    libtool \
    m4 \
    gdb \
    valgrind \
    strace \
    netcat-traditional \
    git \
    # git needs ssh-keygen to make SSH-format commit signatures (gpg.format=ssh)
    # and ssh to push over git@github.com; neither is in the base image
    openssh-client \
    curl \
    wget \
    gnupg \
    ca-certificates \
    apt-transport-https \
    lsb-release \
    procps \
    unzip \
    sudo \
    vim \
    less \
    sqlite3 \
    # bookworm ships no locale but C/POSIX, so anything that honours LANG
    # (perl, which git's tooling shells out to) warns and falls back
    locales \
  && sed -i 's/^# *en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen \
  && locale-gen \
  && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://packages.sury.org/php/apt.gpg -o /usr/share/keyrings/deb.sury.org-php.gpg \
  && echo "deb [signed-by=/usr/share/keyrings/deb.sury.org-php.gpg] https://packages.sury.org/php/ $(lsb_release -sc) main" \
    > /etc/apt/sources.list.d/php.list

# php-dev is installed separately to avoid accidentally pulling in the latest
# php version when installing 7.x
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    php${PHP_VERSION}-cli \
    php${PHP_VERSION}-curl \
    php${PHP_VERSION}-sqlite3 \
    php${PHP_VERSION}-cli-dbgsym \
    php${PHP_VERSION}-common-dbgsym \
  && apt-get install -y --no-install-recommends php${PHP_VERSION}-dev \
  && rm -rf /var/lib/apt/lists/*

# the extension is built into the bind-mounted workspace; point the php
# extension dir at it so `extension=otel` works without a rebuild/reinstall
RUN ln -s /workspace/otel/modules/otel.so $(php-config --extension-dir)/otel.so \
  && cp $(php-config --extension-dir)/build/run-tests.php /home/${USERNAME}/

# docker seeds a new named volume from the image directory it covers, so
# creating this up front gives the cargo target volume the right owner
RUN mkdir -p /workspace/otel/target \
  && chown -R ${USER_UID}:${USER_GID} /workspace

COPY --from=composer /usr/bin/composer /usr/local/bin/composer

# installing an extra apt package or poking at a root-owned file is common
# enough during development to be worth having sudo available
RUN echo "${USERNAME} ALL=(root) NOPASSWD:ALL" > /etc/sudoers.d/${USERNAME} \
  && chmod 0440 /etc/sudoers.d/${USERNAME}

# rust-src lets rust-analyzer resolve std; clippy/rustfmt for the editor
USER ${USERNAME}
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
  | sh -s -- -y --no-modify-path --default-toolchain stable \
      --component rust-src --component rustfmt --component clippy

# these are mount points for named volumes; they must exist (and be owned by
# this user) in the image, because docker seeds a new volume from the image
# directory it covers and otherwise creates it root-owned
RUN mkdir -p /home/${USERNAME}/.cargo/registry \
             /home/${USERNAME}/.cargo/git \
             /home/${USERNAME}/.config/dev-tools
USER root

# LANG is set here rather than left to the editor: vscode's integrated
# terminal only injects the host's locale when LANG is unset or not UTF-8, so
# setting it means every shell gets the locale generated above
ENV PATH="/home/${USERNAME}/.cargo/bin:${PATH}" \
    TEST_PHP_EXECUTABLE="/usr/bin/php" \
    LANG="en_US.UTF-8"

# login shells source /etc/profile, which resets PATH and would drop the ENV
# above, so put cargo on the path there too
RUN printf 'export PATH="/home/%s/.cargo/bin:$PATH"\n' "${USERNAME}" \
      > /etc/profile.d/10-rust.sh \
  && chmod 0755 /etc/profile.d/10-rust.sh

USER ${USERNAME}
