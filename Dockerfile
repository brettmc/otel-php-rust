FROM php:8.4-cli-bullseye

WORKDIR /usr/src/myapp

RUN groupadd -g 1000 php-rust \
  && useradd -g 1000 --create-home php-rust

RUN apt-get update \
  && apt-get install -y llvm-dev libclang-dev

USER php-rust

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y