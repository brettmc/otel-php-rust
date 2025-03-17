FROM php:8.4-fpm-bullseye

WORKDIR /usr/src/myapp

RUN groupadd -g 1000 php-rust \
  && useradd -g 1000 --create-home php-rust

RUN apt-get update \
  && apt-get install -y llvm-dev libclang-dev gdb valgrind netcat

ADD https://github.com/mlocati/docker-php-extension-installer/releases/latest/download/install-php-extensions /usr/local/bin/

RUN ln -s /usr/src/myapp/target/debug/libotel.so $(php-config --extension-dir)/otel.so \
  && cp $(php-config --lib-dir)/php/build/run-tests.php /home/php-rust \
  && chmod +x /usr/local/bin/install-php-extensions \
  && install-php-extensions @composer
  
USER php-rust

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y