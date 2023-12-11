+++
title = "Get started"
weight = 2
+++

# Get started

`gw` is a simple tool you only need a local git repository. In this documentation, we will use `/path/to/repo` as a placeholder. Note that every example is written for Linux, but it should be working similarly on Windows as well.

## How to use gw

To get started, all you have to do is to point `gw` to a local repository. This will run `git fetch` periodically and pull if there are any changes.

```sh
gw /path/to/repo
```

To make it more useful, we can use the `--script` (or `-s`) flag to run commands when there are updates. You can define multiple scripts however for multiple commands or advanced logic, it is recommended to use a `update.sh` file in the repository.

```sh
gw /path/to/repo --script 'echo pulled'
gw /path/to/repo --script 'echo building' --script 'echo deploying'
```

## Use gw as a service

To put this in the background or wrap this in a [systemd unit](/usage/systemd) or [docker container](/usage/docker).

