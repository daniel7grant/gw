+++
title = "Get started"
weight = 2
+++

# Get started

`gw` is a simple program, that you can use to pull changes from a remote repository and run scripts on the change.

## Prerequisites

First, make sure, that `gw` is installed successfully and is in your PATH. If you don't have it, start with [Installation](/usage/installation):

```sh
$ gw --version
0.2.2
```

The other necessary part is a git repository to which you have pull access. It is recommended to use a repository that you know, but if you don't have one at hand, you can use the [daniel7grant/time](https://github.com/daniel7grant/time) repository. This is an example repository that is updated in every minute, so it is useful to test the auto update of `gw`. First clone this repository (if you are using your own, clone again), and enter the cloned directory:

```sh
git clone https://github.com/daniel7grant/time.git
cd time
```

## Pull files automatically

To get started, point `gw` to this local repository. By default it pulls the changes every minute. We can add the `--verbose` or `-v` flag to see when the changes occur:

```sh
gw . -v
```

If you are using your own repository, create a commit in a different place, and see how it gets automatically pulled (in the case of the `time` repo, there is a commit every minute). The verbose logs should print that a git pull happened:

```sh
$ gw . -v
# ...
2024-03-10T14:48:13.447Z DEBUG [gw_bin::checks::git::repository] Checked out fc23d21 on branch main.
2024-03-10T14:48:13.447Z DEBUG [gw_bin::start] There are updates, pulling.
```

Also check the files or the `git log` to see that it the repository has been updated:

```sh
cat DATETIME  # it should contain the latest time
git log -1  # it should be a commit in the last minute
```

## Run scripts on pull

Pulling files automatically is useful but the `--script` or `-s` flag unlocks `gw`'s potential: it can run any kind of custom script if there are any changes. For a simple example, we can print the content of a file to the log with `cat`:

```sh
gw . -v --script 'cat DATETIME'
```

This will run every time there is a new commit, and after the pull it will print the file contents. You can see that the results are printed in the log:

```sh
$ gw . -v --script 'cat DATETIME'
# ...
2024-03-10T15:04:37.740Z DEBUG [gw_bin::start] There are updates, running scripts.
2024-03-10T15:04:37.740Z DEBUG [gw_bin::actions::script] Running script: cat DATETIME in directory /home/grant/Development/quick/time.
2024-03-10T15:04:37.742Z DEBUG [gw_bin::actions::script] Command success, output:
2024-03-10T15:04:37.742Z DEBUG [gw_bin::actions::script]   2024-03-10T15:04:01+0000
```

You can add multiple scripts, which will run one after another. Use these scripts to build source files, restarts deployments and anything else that you can imagine.

## Next steps

If you like `gw`, there are multiple ways to use it for real-life use-cases.

If you want to put the `gw` script in the background, you can:

- wrap into a [systemd unit](/usage/systemd), if you want to manage it with a single file;
- start in a [docker container](/usage/docker), if you already use Docker in your workflow;
- or run periodically with [cron](/usage/crontab), if you don't have shell access to the server.

If you are interested in some ideas on how to use `gw`:

- if you only need to pull files, see [PHP guide](/guides/php);
- if you are using a compiled language, see [Guide for compiled languages](/guides/compiled) for example on restarting a process;
- if you want to use `gw` with a `docker-compose.yaml`, see [Guide for docker-compose](guides/docker-compose);
- if you want to easily manage configuration files as GitOps, see [Configuration guide](/guides/configuration);
- for a full-blown example, check out [Netlify](/guides/netlify);
- and many other things, for the incomplete list [guides page](/guides).
