FROM rust:1.80 AS builder

WORKDIR /app

ARG OPENSSL_STATIC=1

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        make \
        musl-tools \
        perl && \
    rm -rf /var/lib/apt/lists/* && \
    rustup target add x86_64-unknown-linux-musl

COPY ./Cargo.lock ./Cargo.toml /app
COPY ./src /app/src

RUN cargo build --release --target x86_64-unknown-linux-musl


FROM alpine:3.20

RUN apk add --no-cache \
        ca-certificates

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/gw /usr/bin/gw

ENTRYPOINT ["/usr/bin/gw"]
