+++
title = "Env variables"
weight = 4
+++

# Environment variables

The different steps can add variables to the context, which are exposed to the scripts as environment variables.
All of these are prefixed with `GW_` to avoid collisions. The second part usually identifies the specific trigger,
check or action.

If you want to use the environment variables in [command-line arguments](/reference/commandline), make sure to use the subshell variants (`-S`, `-P`),
because only these can expand variables. It is recommended to use single-quotes to avoid expanding at start time. A good way
to debug environment variables is to print them with `-S 'printenv'`.

## Trigger variables

These are the variables that are exposed from the trigger, which can be scheduled trigger or an HTTP endpoint.

| Variable name       | Example            | Notes                                   |
| ------------------- | ------------------ | --------------------------------------- |
| `GW_TRIGGER_NAME`   | `SCHEDULE`, `HTTP` | The identifier of the trigger.          |
| `GW_HTTP_METHOD`    | `GET`, `POST`      | The HTTP method that was called.        |
| `GW_HTTP_URL`       | `/`, `/trigger`    | The HTTP URL that was called.           |
| `GW_SCHEDULE_DELAY` | `1m`, `1d`, `1w`   | The delay between two scheduled checks. |

## Check variables

These are the variables that are exposed from the check, which currently is always git.

| Variable name                    | Example                              | Notes                                         |
| -------------------------------- | ------------------------------------ | --------------------------------------------- |
| `GW_CHECK_NAME`                  | `GIT`                                | The identifier of the check.                  |
| `GW_GIT_BEFORE_COMMIT_SHA`       | `acfd4f88da199...`                   | The SHA of the commit before the pull.        |
| `GW_GIT_BEFORE_COMMIT_SHORT_SHA` | `acfd4f8`                            | The 7-character short hash of the commit.     |
| `GW_GIT_BRANCH_NAME`             | `main`                               | The name of the branch, that the repo is on.  |
| `GW_GIT_COMMIT_SHA`              | `acfd4f88da199...`                   | The SHA of the commit after the pull.         |
| `GW_GIT_COMMIT_SHORT_SHA`        | `acfd4f8`                            | The 7-character short hash of the commit.     |
| `GW_GIT_REF_NAME`                | `refs/heads/main`, `refs/tags/v1.0`  | The full name of the current git ref.         |
| `GW_GIT_REF_TYPE`                | `branch`, `tag`                      | The type of the ref we are currently on.      |
| `GW_GIT_REMOTE_NAME`             | `origin`                             | The name of the remote used.                  |
| `GW_GIT_REMOTE_URL`              | `git@github.com:daniel7grant/gw.git` | The URL to the git remote.                    |
| `GW_GIT_TAG_NAME`                | `v1.0`                               | The tag of the pulled commit if there is one. |

## Action variables

These are the variables added by the action, which is script or process.

| Variable name    | Example             | Notes                                       |
| ---------------- | ------------------- | ------------------------------------------- |
| `GW_ACTION_NAME` | `SCRIPT`, `PROCESS` | The identifier of the action.               |
| `GW_DIRECTORY`   | `/src/http/gw`      | The absolute path to the current directory. |
