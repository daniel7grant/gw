+++
title = "Installation"
weight = 1
+++

# Installation

`gw` is a simple few MB binary, which you can install multiple ways.

## Download from GitHub releases

The easiest way is to download the zipped binary from [Github Releases](https://github.com/daniel7grant/gw/releases) and install it to your path:

```sh
curl -LO https://github.com/daniel7grant/gw/releases/download/v0.2.2/gw-bin_x86_64-unknown-linux-gnu.zip
unzip gw-bin_x86_64-unknown-linux-gnu.zip
mv gw /usr/local/bin/gw
rm gw-bin_x86_64-unknown-linux-gnu.zip
```

## Install with Cargo

The other way is to install the `gw` binary with Cargo. Use [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) for a faster install or `cargo install` will build it from source.

```sh
cargo binstall gw-bin
# or
cargo install gw-bin
```
