+++
title = "Get started"
sort_by = "weight"

insert_anchor_links = "right"
+++

# gw

Watch local git repositories, keep in sync with remote and run commands.

## Motivation

As continuos deployment is getting more of an industry standard, lot of teams are looking for a tool that allows them to only merge to master and have these changes immediately rolled out to live. For a long time, teams were forced to use proprietary hosting (e.g. Netlify or Vercel), build their own push-based CI pipelines (e.g. on GitHub Actions or GitLab Ci) or use heavyweight pull-based architecture (e.g. ArgoCD). It is especially difficult with deployments on-prem or behind a VPN.  `gw` is a lightweight tool that runs on the servers directly, pulls if there are changes in the remote repo and runs commands. You can run a build command, restart your deployment or anything you can imagine.

## Get started

The easiest way is to download the zipped binary from [Github Releases](https://github.com/daniel7grant/gw/releases) or install with cargo:

```sh
cargo install gw-bin
```

For more installation methods, see [Installation](./usage/installation).

## Usage

To use `gw`, you have to point it to your local repository and it will pull changes automatically. You can run scripts on every pull to build or restart your deployments with the `--script` flag:

```sh
gw /path/to/repo --script 'tool run build' --script 'deployment restart'
```

