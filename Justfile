#!/usr/bin/env just --justfile

lint:
  taplo format && \
  cargo +nightly fmt && \
  cargo fix --workspace --allow-dirty --allow-staged --broken-code && \
  cargo clippy --fix --workspace --allow-dirty --allow-staged