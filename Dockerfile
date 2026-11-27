FROM composer:lts AS composer
FROM debian:bookworm

ENV DEBIAN_FRONTEND=noninteractive

# the repository is mounted at /workspace and the extension lives one level
# down; this matches the dev containers in .devcontainer/
WORKDIR /workspace/otel

RUN groupadd -g 1000 php-rust \
  && useradd -g 1000 --create-home php-rust

# build toolchain for phper/bindgen, plus gdb and valgrind for debugging the
# extension inside php
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    llvm-dev \
    libclang-dev \
    build-essential \
    pkg-config \
    gdb \
    valgrind \
    curl \
    ca-certificates \
    lsb-release \
    unzip \
  && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://packages.sury.org/php/apt.gpg -o /usr/share/keyrings/deb.sury.org-php.gpg \
  && echo "deb [signed-by=/usr/share/keyrings/deb.sury.org-php.gpg] https://packages.sury.org/php/ $(lsb_release -sc) main" \
    > /etc/apt/sources.list.d/php.list

USER php-rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
USER root

COPY --from=composer /usr/bin/composer /usr/local/bin/composer

ENV PATH="/home/php-rust/.cargo/bin:${PATH}" \
    TEST_PHP_EXECUTABLE="/usr/bin/php"

ARG PHP_VERSION=8.4

# php-dev installed separately to avoid accidental install of latest php version when installing 7.x :(
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    php${PHP_VERSION}-cli \
    php${PHP_VERSION}-curl \
    php${PHP_VERSION}-cli-dbgsym \
    php${PHP_VERSION}-common-dbgsym \
    php${PHP_VERSION}-sqlite3 \
    sqlite3 \
  && apt-get install -y --no-install-recommends php${PHP_VERSION}-dev \
  && rm -rf /var/lib/apt/lists/* \
  && ln -s /workspace/otel/modules/otel.so $(php-config --extension-dir)/otel.so

RUN cp $(php-config --extension-dir)/build/run-tests.php /home/php-rust/

USER php-rust
