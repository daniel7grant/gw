FROM rust:1.80 AS builder

WORKDIR /app

ARG OPENSSL_STATIC=1

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libc6-dev \
        libssl-dev \
        make \
        musl-tools \
        perl && \
    rm -rf /var/lib/apt/lists/* && \
    # Install the musl equivalent of the default target to work on arm
    GNU_TARGET="$(rustup target list --installed)" && \
    MUSL_TARGET="$(echo $GNU_TARGET | sed 's/gnu/musl/')" && \
    rustup target add $MUSL_TARGET

COPY ./Cargo.lock ./Cargo.toml /app
COPY ./src /app/src

# Change the default target to work on arm, but maintain release
RUN MUSL_TARGET="$(rustup target list --installed | grep musl)" && \
    cargo build --release --target $MUSL_TARGET && \
    cp -r target/$MUSL_TARGET/release/* target/release


FROM alpine:3.20

RUN apk add --no-cache \
        ca-certificates

COPY --from=builder /app/target/release/gw /usr/bin/gw

ENTRYPOINT ["/usr/bin/gw"]
