+++
title = "Actions on pull"
weight = 3
+++

# Actions on pull

The main point of `gw` is to do actions every time there code is pulled. There are multiple actions: running a script or restarting a background process. The order of the actions matter, and they will be executed sequentially based on the order of the command-line arguments.

## Scripts

The simplest action is to run a command on pull with `--script` or `-s`:

```sh
gw /path/to/repo -s 'echo "updated"'
```

You can define multiple scripts, these will run one after another (there is currently no way to parallelise these). If one of the scripts fail, the other scripts won't run at all. You can use scripts to run tests before updating your code.

```sh
gw /path/to/repo -s 'echo "testing"' -s 'echo "updating"'
```

> **Note**: If you have more than 2-3 scripts on every pull it might be worth it to refactor it into an `update.sh` shell script. It can contain logic and be commited which helps if you want to change it without updating the gw process.

The output of the script is not printed by default, you can increase verbosity (`-v`) to get output from the script:

```sh
$ gw /path/to/repo -v -s 'echo "updated"'
2024-10-18T16:28:53.907Z [INFO ] There are updates, running actions.
2024-10-18T16:28:53.907Z [INFO ] Running script "echo" in /path/to/repo.
2024-10-18T16:28:53.913Z [DEBUG] [echo] updated
2024-10-18T16:28:53.913Z [INFO ] Script "echo" finished successfully.
```

By default, scripts are executed directly to avoid common issues with shells (e.g. shell injection and unexpected globbing). If you instead want to run in a shell to expand variables or use shell specific functionality (e.g. pipes or multiple commands), use the `-S` flag. These scripts will run in a shell: `/bin/sh` on Linux and `cmd.exe` on Windows.

```sh
gw /path/to/repo -S 'ls -l . | wc -l'
```

The full enviroment is passed to scripts with a number of [gw-specific environment variables](/reference/environment-variables). If you want to use variables make sure to use singlequotes so they aren't expanded beforehand.

```sh
gw /path/to/repo -S 'ls -l $BUILD_DIRECTORY | wc -l'
```

Best use-cases for scripts:

-   [compile](/guides/compiled) or transpile your code,
-   rebuild some assets,
-   restart or reload a separately running program.

## Processes

If you have some long-running program, you can use `--process` or `-p` to start as a background process and `gw` will restart it on every pull:

```sh
gw /path/to/repo -p 'ping 1.1.1.1'
```

Processes are started when `gw` is started and they are kept in the background. If there is a change the process is stopped and a new process is started. If you want to look at the output of process, you have to increase verbosity (`-v`):

```sh
$ gw /path/to/repo -v -s 'ping 1.1.1.1'
2024-03-10T15:04:37.740Z [INFO ] There are updates, running actions.
2024-10-16T18:04:25.888Z [INFO ] Starting process "ping" in /path/to/repo.
2024-10-16T18:04:25.906Z [DEBUG] [ping] PING 1.1.1.1 (1.1.1.1) 56(84) bytes of data.
2024-10-16T18:04:25.906Z [DEBUG] [ping] 64 bytes from 1.1.1.1: icmp_seq=1 ttl=57 time=16.8 ms
```

Similarly to scripts, processes are executed directly. If you want to use the native shell for variable expansion or shell-specific functionality, you can use `-P`.

```sh
gw /path/to/repo -P 'ping $TARGET_IP'
```

Unlike scripts, you can only define one process. Processes also can't access gw-specific environment variables. Scripts defined before process will be run before the restart and if defined after they will run after. If any of the scripts before the process fails the process will not be restarted. You can add tests and other checks to only restart the process if the code is 100% correct.

```sh
gw /path/to/repo -s 'echo this runs before' -p 'ping 1.1.1.1' -s 'echo this runs after'
```

If a process fails, by default it marked failed and an error printed. If you want to retry the process you can set the `--process-retries` flag:

```sh
gw /path/to/repo -v -s 'ping 1.1.1.1' --process-retries 5
```

You can also change the stopping behaviour. By default processes are first tried to be gracefully stopped with SIGINT and after some timeout (default: 10s) they are killed. If you want to influence these values you can set `--stop-signal` and `--stop-timeout` respectively. On non-Unix systems these options do nothing and the process is always killed.

```sh
gw /path/to/repo -v -s 'ping 1.1.1.1' --stop-signal SIGTERM --stop-timeout 10s
```

Best use-cases for processes:

-   run [interpreted programs](/guides/interpreted) e.g. web frameworks,
-   run binaries after [compiling](/guides/compiled),
-   run external programs to restart [on config change](/guides/configuration).
