FROM rust:1.75 AS builder

WORKDIR /app

#RUN apk add musl-dev libressl-dev 

COPY ./Cargo.lock ./Cargo.toml /app
COPY ./src /app/src

RUN cargo build --release


FROM debian:12-slim

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        openssl \
        openssh-client \
        && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/gw /usr/bin/gw

ENTRYPOINT ["/usr/bin/gw"]
