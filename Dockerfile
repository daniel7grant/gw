# Reproduce some amount of rust:1.80-alpine because it doesn't support armv7 now
FROM alpine:3.20 AS rust-1.80-alpine

RUN apk add --no-cache \
        ca-certificates \
        curl \
        gcc

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --no-modify-path --profile minimal --default-toolchain 1.80


FROM rust-1.80-alpine AS builder

WORKDIR /app

ARG OPENSSL_STATIC=1

RUN apk add --no-cache \
        make \
        musl-dev \
        perl

COPY ./Cargo.lock ./Cargo.toml /app
COPY ./src /app/src

RUN cargo build --release


FROM alpine:3.20

RUN apk add --no-cache \
        ca-certificates

COPY --from=builder /app/target/release/gw /usr/bin/gw

ENTRYPOINT ["/usr/bin/gw"]
