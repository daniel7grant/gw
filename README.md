# gw

Watch local git repositories, keep in sync with remote and run commands.

## Motivation

`gw` is a lightweight binary that manages a simple pull-based continuous deployment for you. It watches a local git repository, fetches if the remote changes, and builds or deploys your code. Current CD solutions either lock you into proprietary software (e.g. Netlify or Vercel) or complicated to run and manage (e.g. ArgoCD). `gw` is a service that can run everywhere (even behind NAT or VPN), synchronizes code with your remote and deploys immediately, saving your developers time and energy.

Features of `gw`:
- **lightweight**: it is only a 1.5MB binary (~7MB with git and ssh statically built-in)
- **runs anywhere**: use it on baremetal, [systemd](https://gw.danielgrants.com/usage/systemd.md) or [docker](https://gw.danielgrants.com/usage/docker.md)
- **open source**: written entirely in Rust, you can build it from source in a few minutes
- **pull-based**: works on any network, even behind a NAT or VPN
- **flexible**: build, deploy, restart or anything you can imagine

## Installation

To get started with `gw`, you can use the install script:

```sh
curl https://gw.danielgrants.com/install.sh | sh
```

For more installation methods, see the [documentation](https://gw.danielgrants.com/usage/installation/).

## Get started

`gw` is a simple program, that you can use to pull changes from a remote repository and run scripts on the change.

## Prerequisites

First, make sure, that `gw` is installed successfully and is in your PATH:

```sh
$ gw --version
0.3.2
```

The other necessary part is a git repository to which you have pull access. It is recommended to use a repository that you know, but if you don't have one at hand, you can use the [daniel7grant/time](https://github.com/daniel7grant/time) repository. This is an example repository that is updated in every minute, so it is useful to test the auto update of `gw`. First clone this repository (if you are using your own, clone again), and enter the cloned directory:

```sh
git clone https://github.com/daniel7grant/time.git
cd time
```

### Pull files automatically

To get started, point `gw` to this local repository. By default it pulls the changes every minute. We can add the `--verbose` or `-v` flag to see when the changes occur:

```sh
gw /path/to/repo -v
```

If you are using your own repository, create a commit in a different place, and see how it gets automatically pulled (in the case of the `time` repo, there is a commit every minute). The verbose logs should print that a git pull happened:

```sh
$ gw /path/to/repo -v
# ...
2024-03-10T14:48:13.447Z DEBUG [gw_bin::checks::git::repository] Checked out fc23d21 on branch main.
2024-03-10T14:48:13.447Z INFO  [gw_bin::start] There are updates, pulling.
```

Also check the files or the `git log` to see that it the repository has been updated:

```sh
cat DATETIME  # it should contain the latest time
git log -1  # it should be a commit in the last minute
```

### Run scripts on pull

Pulling files automatically is useful but the `--script` or `-s` flag unlocks `gw`'s potential: it can run any kind of custom script if there are any changes. For a simple example, we can print the content of a file to the log with `cat`:

```sh
gw /path/to/repo -v --script 'cat DATETIME'
```

This will run every time there is a new commit, and after the pull it will print the file contents. You can see that the results are printed in the log:

```sh
$ gw /path/to/repo -v --script 'cat DATETIME'
# ...
2024-10-18T16:28:53.907Z INFO  [gw_bin::start] There are updates, running actions.
2024-10-18T16:28:53.907Z INFO  [gw_bin::actions::script] Running script "cat" in /path/to/repo.
2024-10-18T16:28:53.913Z DEBUG [gw_bin::actions::script] [cat] 2024-10-18T16:28:00+0000
2024-10-18T16:28:53.913Z INFO  [gw_bin::actions::script] Script "cat" finished successfully.
```

You can add multiple scripts, which will run one after another. Use these scripts to build source files, restarts deployments and anything else that you can imagine.

### Run subprocess, restart on pull

It is often enough to run scripts, but many times you also want to maintain a long-running process e.g. for web services. `gw` can help you with this, using the `-p` flag. This will start a process in the background and restart it on pull.

For example starting a python web server:

```sh
$ gw /path/to/repo -v -p "python -m http.server"
# ...
2024-10-06T21:58:21.306Z DEBUG [gw] Setting up ProcessAction "python -m http.server" on change.
2024-10-06T21:58:21.306Z DEBUG [gw_bin::actions::process] Starting process: "python" in directory /path/to/repo.
2024-10-06T21:58:56.211Z DEBUG [gw_bin::actions::process] [python] Serving HTTP on 0.0.0.0 port 8000 (http://0.0.0.0:8000/) ...
```

This will run a python process in the background and stop and start it again if a git pull happened. Just wrap your deployment script with `gw` and see it gets updated every time you push to git.

## Next steps

If you like `gw`, there are multiple ways to use it for real-life use-cases.

If you want to put the `gw` script in the background, you can:

- wrap into a [systemd unit](https://gw.danielgrants.com/usage/systemd), if you want to manage it with a single file;
- start in a [docker container](https://gw.danielgrants.com/usage/docker), if you already use Docker in your workflow;
- or run periodically with [cron](https://gw.danielgrants.com/usage/crontab), if you don't have shell access to the server.

If you are interested in some ideas on how to use `gw`:

- if you only need to pull files, see [PHP guide](https://gw.danielgrants.com/guides/php);
- if you are using a dynamic language (e.g. JavaScript, Python, Ruby), see [Guide for dynamic languages](https://gw.danielgrants.com/guides/dynamic) for example on running your process;
- if you are using a compiled language (e.g. TypeScript, Go, Rust), see [Guide for compiled languages](https://gw.danielgrants.com/guides/compiled) for example on compiling your program;
- if you use a `docker-compose.yaml`, see [Guide for docker-compose](guides/docker-compose);
- if you want to easily manage configuration files as GitOps, see [Configuration guide](https://gw.danielgrants.com/guides/configuration);
- for a full-blown example, check out [Netlify](https://gw.danielgrants.com/guides/netlify);
- and many other things, for the incomplete list [guides page](https://gw.danielgrants.com/guides).
