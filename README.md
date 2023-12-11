# gw

Watch local git repositories, keep in sync with remote and run commands.

## Motivation

As continuos deployment is getting more of an industry standard, lot of teams are looking for a tool that allows them to only merge to master and have these changes immediately rolled out to live. For a long time, teams were forced to use proprietary hosting (e.g. Netlify or Vercel), build their own push-based CI pipelines (e.g. on GitHub Actions or GitLab Ci) or use heavyweight pull-based architecture (e.g. ArgoCD). It is especially difficult with deployments on-prem or behind a VPN.  `gw` is a lightweight tool that runs on the servers directly, pulls if there are changes in the remote repo and runs commands. You can run a build command, restart your deployment or anything you can imagine.

## Get started

To get started install the `gw` binary:

```sh
cargo binstall gw-bin
# or
cargo install gw-bin
```

## Usage

All you have to do is point to a local repository and add a script to run, if there is an update.

To get started, all you have to do is to point `gw` to a local repository. This will run `git fetch` periodically and pull if there are any changes.

```sh
gw /path/to/repo
```

To make it more useful, we can use the `--script` (or `-s`) flag to run commands when there are updates. You can define multiple scripts however for multiple commands or advanced logic, it is recommended to use a `update.sh` file in the repository.

```sh
gw /path/to/repo --script 'echo pulled'
gw /path/to/repo --script 'echo building' --script 'echo deploying'
```

To put this in the background simply wrap this in a [systemd unit](./docs/systemd.md) or [docker container](./docs/docker.md).

But this is not all `gw` can do. With a little creativity you can create a lot of things, for example:

- pull changes for [development](./docs/development.md) and get a notification;
- rollout a [docker-compose](./docs/docker-compose.md) deployment continously;
- build on all commits for a minimal [Netlify](./docs/netlify.md) alternative;
- build on tags for a versioned [documentation website](./docs/documentation-tags.md),

...and many thing else. For a complete list, check out the [recipes page](./docs/recipes.md).

