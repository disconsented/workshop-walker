FROM rust:1.86.0 AS build-rust
LABEL authors="disconsented"
RUN apt-get update && apt-get install -y npm libclang-dev
WORKDIR /usr/src/workshop-walker
COPY src/ /usr/src/workshop-walker/src/
COPY migrations/ /usr/src/workshop-walker/migrations/
COPY Cargo.lock Cargo.toml /usr/src/workshop-walker/
RUN cargo build --release

FROM node:23 AS build-node
COPY ui/ /usr/src/workshop-walker/ui/
RUN cd /usr/src/workshop-walker/ui && npm i && npm run build && ls -lah

FROM  gcr.io/distroless/cc-debian12:latest
COPY --from=build-rust  /usr/src/workshop-walker/target/release/workshop-walker /
COPY --from=build-node /usr/src/workshop-walker/ui/build/ /ui/build/
CMD ["./workshop-walker"]