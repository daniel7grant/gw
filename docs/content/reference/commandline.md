+++
title = "CLI arguments"
weight = 3
+++

# Command-line arguments

This details the arguments that the `gw` binary takes.

## Positional arguments

Every `gw` execution should specify a directory to a git repository. This will be the repository which the `gw` checks to see if there are any changes and run actions.

## Flag arguments

`gw` follows GNU argument conventions, so every short arguments start with `-` and long arguments start with `--`.

### Basic flags

To get information about `gw` or change the output settings (more verbose with `-v` or quieter with `-q`), you can use these flags:

| Argument name     | Example     | Notes                                                                  |
| ----------------- | ----------- | ---------------------------------------------------------------------- |
| `-v`, `--verbose` | `-v`, `-vv` | Increase verbosity, can be set multiple times (-v debug, -vv tracing). |
| `-q`, `--quiet`   | `-q`        | Only print error messages.                                             |
| `-V`, `--version` | `--version` | Print the current version.                                             |
| `-h`, `--help`    | `--help`    | Print this help.                                                       |

### Trigger flags

Use these flags, to set the different modes to check for changes:

-   Scheduled triggers (`-d`, default every 1 minute): check with a specified interval using [duration-string](https://github.com/Ronniskansing/duration-string) settings. Pass `0s` for disabling scheduled triggers.
-   Trigger once (`--once`): check if there are changes and then exit immediately.
-   Http trigger (`--http`): run an HTTP server on an interface and port (e. g. `0.0.0.0:8000`), which trigger on any incoming request. For more information, see [Webhook](/usage/webhook).

| Argument name   | Example                                          | Notes                                                                  |
| --------------- | ------------------------------------------------ | ---------------------------------------------------------------------- |
| `-d`, `--every` | `-d 5m`, `-d 1h`, `-d 0s`                        | Refreshes the repo with this interval. (default: 1m)                   |
| `--once`        | `--once`                                         | Try to pull only once. Useful for cronjobs.                            |
| `--http`        | `--http localhost:1234`, `--http 127.0.0.1:4321` | Runs an HTTP server on the URL, which allows to trigger by calling it. |

### Check flags

These flags change the way `gw` checks the git repository for changes:

-   On every push (`--on push`, default): pull the commits on the current branch and run actions if there are any new commits.
-   On every tag (`--on tag` or `--on tag:v*`): fetch the commits on the current branch and only pull to the first tag. You can pass a glob, in which case the first tag matching the glob. If there are no matching tags, no pull happens.

You can also configure the authentication for the git repository:

-   SSH authentication (`-i`, `--ssh-key`): specify the path to the SSH key that will.
-   HTTP authentication (`--git-username`, `--git-token`): specify a username-token pair.

For more information see [Authentication](/reference/authentication).

| Argument name      | Example                                                | Notes                                                                                           |
| ------------------ | ------------------------------------------------------ | ----------------------------------------------------------------------------------------------- |
| `--on`             | `--on push`, `--on tag`, `--on tag:v*`                 | The trigger on which to run (can be `push`, `tag` or `tag:pattern`). (default: push)            |
| `-i`, `--ssh-key`  | `-i ~/.ssh/test.id_rsa`                                | Set the path for an ssh-key to be used when pulling.                                            |
| `--git-username`   | `--git-username daniel7grant`                          | Set the username for git to be used when pulling with HTTPS.                                    |
| `--git-token`      | `--git-token 'ghp_jB3c5...'`                           | Set the token for git to be used when pulling with HTTPS.                                       |
| `--git-known-host` | `--git-known-host 'example.com ssh-rsa AAAAB3NzaC...'` | Add this line to the known_hosts file to be created (e.g. "example.com ssh-ed25519 AAAAC3..."). |

### Action flags

These flags configure the actions that should be run when the changes occur. These come in two flavours lowercase letters indicate that it is run directly, while uppercase letters will run in a subshell (e.g. `/bin/sh` on Linux). This is useful if you want to expand variables, pipe commands etc. It is recommended to always use single-quotes for the argument values to avoid accidental shell issues (e.g. expanding variables at start time).

-   Run scripts (`-s`, `-S`): execute a script on every change, that will be waited until it ends.
-   Start process (`-p`, `-P`): start a process, when starting `gw`, that will be restarted on every change.

You can also configure the process running:

-   Retries (`--process-retries`): in case of a failed process, how many time should it be restarted, before marking it failed.
-   Stop settings (`--stop-signal`, `--stop-timeout`): how to stop the process in case of a restart, by default sending `SIGINT` and after 10s a `SIGKILL` (supported only on `*NIX`).

For more information see [Actions on pull](/usage/actions).

| Argument name       | Example             | Notes                                                                                                                       |
| ------------------- | ------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `-s`, `--script`    | `-s 'cat FILENAME'` | A script to run on changes, you can define multiple times.                                                                  |
| `-S`                |                     | Run a script in a shell.                                                                                                    |
| `-p`, `--process`   |                     | A background process that will be restarted on change.                                                                      |
| `-P`                |                     | Run a background process in a shell.                                                                                        |
| `--process-retries` |                     | The number of times to retry the background process in case it fails. By default 0 for no retries.                          |
| `--stop-signal`     |                     | The stop signal to give the background process. Useful for graceful shutdowns. By default SIGINT. (Only supported on \*NIX) |
| `--stop-timeout`    |                     | The timeout to wait before killing for the background process to shutdown gracefully. By default 10s.                       |
