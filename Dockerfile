FROM composer:lts AS composer
FROM debian:bullseye

ENV DEBIAN_FRONTEND=noninteractive

WORKDIR /usr/src/myapp

RUN groupadd -g 1000 php-rust \
  && useradd -g 1000 --create-home php-rust

RUN apt-get update \
  && apt-get install -y llvm-dev libclang-dev gdb valgrind netcat-traditional vim less wget gnupg curl procps strace unzip

RUN apt-get update && apt-get install -y lsb-release apt-transport-https ca-certificates \
  && echo "deb https://packages.sury.org/php/ $(lsb_release -sc) main" > /etc/apt/sources.list.d/php.list \
  && wget -qO - https://packages.sury.org/php/apt.gpg | apt-key add - \
  && apt-get update

USER php-rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
USER root

COPY --from=composer /usr/bin/composer /usr/local/bin/composer

ENV PATH="/home/php-rust/.cargo/bin:${PATH}" \
    TEST_PHP_EXECUTABLE="/usr/bin/php"

ARG PHP_VERSION=8.4

# php-dev installed separately to avoid accidental install of latest php version when installing 7.x :(
RUN apt-get update \
  && apt-get install -y \
    php${PHP_VERSION}-cli \
    php${PHP_VERSION}-cli-dbgsym \
    php${PHP_VERSION}-common-dbgsym \
  && apt-get install -y php${PHP_VERSION}-dev \
  && ln -s /usr/src/myapp/modules/otel.so $(php-config --extension-dir)/otel.so \
  && find /usr/lib/php/ -type f -name run-tests.php -exec cp {} /home/php-rust \;

USER php-rust
