+++
title = "Installation"
weight = 1
+++

# Installation

`gw` is a simple few MB binary, which you can install multiple ways.

## Install from script

The simplest way is to run the installation script:

```sh
curl https://gw.danielgrants.com/install.sh | sh
```

This will download the script to `~/.local/bin` or if run by root to `/usr/local/bin`.

## Download from GitHub releases

Another way is to download the zipped binary from [Github Releases](https://github.com/daniel7grant/gw/releases) and install it to your path:

```sh
curl -LO https://github.com/daniel7grant/gw/releases/download/v0.4.1/gw-bin_x86_64-unknown-linux-gnu.zip
unzip gw-bin_x86_64-unknown-linux-gnu.zip
mv gw ~/.local/bin/gw
rm gw-bin_x86_64-unknown-linux-gnu.zip
```

## Install with Cargo

If you have Rust on your machine, you can also install the `gw` binary with Cargo. Use [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) for a faster install or `cargo install` will build it from source.

```sh
cargo binstall gw-bin
# or
cargo install gw-bin
```
