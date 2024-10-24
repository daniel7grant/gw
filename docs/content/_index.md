+++
title = "Get started"
sort_by = "weight"

insert_anchor_links = "right"
+++

# gw

Watch local git repositories, keep in sync with remote and run commands.

## Motivation

`gw` is a lightweight binary that manages a simple pull-based continuous deployment for you. It watches a local git repository, fetches if the remote changes, and builds or deploys your code. Current CD solutions either lock you into proprietary software (e.g. Netlify or Vercel) or complicated to run and manage (e.g. ArgoCD). `gw` is a service that can run everywhere (even behind NAT or VPN), synchronizes code with your remote and deploys immediately, saving your developers time and energy.

Features of `gw`:
- **lightweight**: it is only a 1.5MB binary (~7MB with git and ssh statically built-in)
- **runs anywhere**: use it on baremetal or [systemd](https://gw.danielgrants.com/usage/systemd.md) on Linux (x86_64 and ARM) or in [Docker](https://gw.danielgrants.com/usage/docker.md) (Windows and MacOS is supported on a best-effort basis)
- **open source**: written entirely in Rust, you can build it from source in a few minutes
- **pull-based**: works on any network, even behind a NAT or VPN
- **flexible**: build, deploy, restart or anything you can imagine

If you want to see how `gw` compare to other products: look at the [comparisons](/reference/comparison).

## Installation

To get started with `gw`, you can use the install script:

```sh
curl https://gw.danielgrants.com/install.sh | sh
```

For more installation methods, see [Installation](/usage/installation).

## Get started

To use `gw`, you have to point it to your local repository and it will pull changes automatically. You can run scripts on every pull to build with the `--script` (or `-s`) flag or run your deployments with the `--process` (or `-p`) flag.

```sh
gw /path/to/repo --script 'run build process' --process 'run deployment'
```

For your first steps with `gw`, see [Get started](/usage/start).

## Next steps

But this is not all `gw` can do. With a little creativity you can create a lot of things, for example:

- pull changes for [development](/guides/development) and get a notification;
- rollout a [docker-compose](/guides/docker-compose) deployment continously;
- build on all commits for a minimal [Netlify](/guides/netlify) alternative,

...and many thing else. For a complete list, check out the [guides page](/guides).
