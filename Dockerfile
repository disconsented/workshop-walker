FROM rust:1.98.1 AS build-rust
LABEL authors="disconsented"
RUN apt-get update && apt-get install -y npm libclang-dev
WORKDIR /usr/src/workshop-walker
COPY src/ /usr/src/workshop-walker/src/
COPY macros/ /usr/src/workshop-walker/macros/
COPY migrations/ /usr/src/workshop-walker/migrations/
COPY migrations-tool/ /usr/src/workshop-walker/migrations-tool/
COPY proc-macros/ /usr/src/workshop-walker/proc-macros/
COPY serde-hack/ /usr/src/workshop-walker/serde-hack/
COPY Cargo.lock Cargo.toml /usr/src/workshop-walker/
RUN cargo build --release --all

FROM node:26 AS build-node
COPY ui/ /usr/src/workshop-walker/ui/
RUN cd /usr/src/workshop-walker/ui && npm i && npm run build && ls -lah

FROM  gcr.io/distroless/cc-debian12:latest
COPY --from=build-rust  /usr/src/workshop-walker/target/release/workshop-walker /
COPY --from=build-node /usr/src/workshop-walker/ui/build/ /ui/build/
COPY migrations/ /migrations/
COPY prompts/ /prompts/
CMD ["./workshop-walker"]