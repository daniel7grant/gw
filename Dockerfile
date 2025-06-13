FROM rust:1.87-alpine AS builder

WORKDIR /app

ARG OPENSSL_STATIC=1

RUN apk add --no-cache \
        make \
        musl-dev \
        perl

COPY ./Cargo.lock ./Cargo.toml /app/
COPY ./src /app/src

RUN cargo build --release


FROM alpine:3.22

RUN apk add --no-cache \
        ca-certificates

COPY --from=builder /app/target/release/gw /usr/bin/gw

ENTRYPOINT ["/usr/bin/gw"]
