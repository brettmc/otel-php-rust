FROM debian:bullseye

WORKDIR /usr/src/myapp

RUN groupadd -g 1000 php-rust \
  && useradd -g 1000 --create-home php-rust

RUN apt-get update \
  && apt-get install -y llvm-dev libclang-dev gdb valgrind netcat-traditional vim less wget gnupg curl

RUN apt-get update && apt-get install -y lsb-release apt-transport-https ca-certificates \
  && echo "deb https://packages.sury.org/php/ $(lsb_release -sc) main" > /etc/apt/sources.list.d/php.list \
  && wget -qO - https://packages.sury.org/php/apt.gpg | apt-key add - \
  && apt-get update

USER php-rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
USER root

ENV PATH="/home/php-rust/.cargo/bin:${PATH}" \
    TEST_PHP_EXECUTABLE="/usr/bin/php"

ARG PHP_VERSION=8.4

RUN apt-get install -y php${PHP_VERSION}-cli php${PHP_VERSION}-dev
RUN ln -s /usr/src/myapp/modules/otel.so $(php-config --extension-dir)/otel.so

RUN wget https://getcomposer.org/download/latest-stable/composer.phar -O /usr/local/bin/composer \
  && chmod +x /usr/local/bin/composer \
  && find /usr/lib/php/ -type f -name run-tests.php -exec cp {} /home/php-rust \;

USER php-rust
