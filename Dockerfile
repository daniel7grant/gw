FROM rust:1.80 AS builder

WORKDIR /app

COPY ./Cargo.lock ./Cargo.toml /app
COPY ./src /app/src

RUN cargo build --release


FROM debian:12-slim

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/gw /usr/bin/gw

ENTRYPOINT ["/usr/bin/gw"]
