#!/usr/bin/env just --justfile

lint:
  taplo format && \
  cargo +nightly fmt && \
  cargo fix --workspace --allow-dirty --allow-staged --broken-code && \
  cargo clippy --fix --workspace --allow-dirty --allow-staged

build_image:
  docker buildx build -t workshop-walker:latest -o type=docker,dest=- . > workshop-image