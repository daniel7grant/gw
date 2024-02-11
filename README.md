# gw

Watch local git repositories, keep in sync with remote and run commands.

## Motivation

`gw` is a lightweight binary that manages a simple pull-based continuous deployment for you. It watches a local git repository, fetches if the remote changes, and builds or deploys your code. Current CD solutions either lock you into proprietary software (e.g. Netlify or Vercel) or complicated to run and manage (e.g. ArgoCD). `gw` is a service that can run everywhere (even behind NAT or VPN), synchronizes code with your remote and deploys immediately, saving your developers time and energy.

Features of `gw`:
- **lightweight**: it is only a 1.4MB binary
- **runs anywhere**: use it on baremetal, [systemd](./docs/content/usage/systemd.md) or [docker](./docs/content/usage/docker.md)
- **open source**: written entirely in Rust, you can build it from source in a few minutes
- **pull-based**: works on any network, even behind a NAT or VPN
- **flexible**: build, deploy, restart or anything you can imagine

## Get started

To get started download the `gw` binary from [releases](https://github.com/daniel7grant/gw/releases/latest) or install with [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```sh
cargo binstall gw-bin
# or
cargo install gw-bin
```

## Usage

All you have to do is point to a local repository and add a script to run if there is an update.

To get started, all you have to do is to point `gw` to a local repository. This will run `git fetch` periodically and pull if there are any changes.

```sh
gw /path/to/repo
```

To make it more useful, we can use the `--script` (or `-s`) flag to run commands when there are updates. You can define multiple scripts however for multiple commands or advanced logic, it is recommended to use a `update.sh` file in the repository.

```sh
gw /path/to/repo --script 'echo pulled'
gw /path/to/repo --script 'echo building' --script 'echo deploying'
```

To put this in the background simply wrap this in a [systemd unit](./docs/content/usage/systemd.md) or [docker container](./docs/content/usage/docker.md).

But this is not all `gw` can do. With a little creativity you can create a lot of things, for example:

- pull changes for [development](./docs/content/guides/development.md) and get a notification;
- rollout a [docker-compose](./docs/content/guides/docker-compose.md) deployment continously;
- build on all commits for a minimal [Netlify](./docs/content/guides/netlify.md) alternative,

...and many thing else. For a complete list, check out the [guides page](./docs/content/guides).

