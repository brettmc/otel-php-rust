FROM php:8.4-fpm-bullseye

WORKDIR /usr/src/myapp

RUN groupadd -g 1000 php-rust \
  && useradd -g 1000 --create-home php-rust

RUN apt-get update \
  && apt-get install -y llvm-dev libclang-dev gdb valgrind netcat

RUN ln -s /usr/src/myapp/target/debug/libotel.so $(php-config --extension-dir)/otel.so \
  && cp $(php-config --lib-dir)/php/build/run-tests.php /home/php-rust

USER php-rust

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y