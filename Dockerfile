FROM rust:1.98.1 AS build-rust
LABEL authors="disconsented"
RUN apt-get update && apt-get install -y --no-install-recommends libclang-dev

WORKDIR /usr/src/workshop-walker
COPY src/ /usr/src/workshop-walker/src/
COPY macros/ /usr/src/workshop-walker/macros/
COPY migrations/ /usr/src/workshop-walker/migrations/
COPY migrations-tool/ /usr/src/workshop-walker/migrations-tool/
COPY proc-macros/ /usr/src/workshop-walker/proc-macros/
COPY serde-hack/ /usr/src/workshop-walker/serde-hack/
COPY Cargo.lock Cargo.toml /usr/src/workshop-walker/
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/workshop-walker/target \
    cargo build --release --all && cp target/release/workshop-walker /workshop-walker

FROM node:26 AS build-node
COPY ui/ /usr/src/workshop-walker/ui/
RUN cd /usr/src/workshop-walker/ui && npm i && npm run build

FROM  gcr.io/distroless/cc-debian13:latest AS runner
COPY --from=build-rust  /workshop-walker /
COPY --from=build-node /usr/src/workshop-walker/ui/build/ /ui/build/
COPY migrations/ /migrations/
COPY prompts/ /prompts/
CMD ["./workshop-walker"]