+++
title = "Actions on pull"
weight = 3
+++

# Actions on pull

The main point of `gw` is to do actions every time there code is pulled. There are multiple actions: running a script or restarting a background process.

## Scripts

The simplest action is to run a command on pull with `--script` or `-s`:

```sh
gw /path/to/repo -s 'echo "updated"'
```

You can define multiple scripts, these will run one after another (there is currently no way to parallelise these). The output of the script is not printed by default, you can increase verbosity (`-v`) to get output from the script:

```sh
$ gw /path/to/repo -v -s 'echo "updated"'
2024-03-10T15:04:37.740Z INFO  [gw_bin::start] There are updates, running actions.
2024-03-10T15:04:37.740Z DEBUG [gw_bin::actions::script] Running script: echo "updated" in directory /path/to/repo.
2024-03-10T15:04:37.742Z DEBUG [gw_bin::actions::script] Command success, output:
2024-03-10T15:04:37.742Z DEBUG [gw_bin::actions::script]   update
```

Scripts are always run first of all actions and run in a shell (`/bin/sh` on Linux and `cmd` on Windows). This way you can expand variables and use shell features e.g. pipes or multiple commands. The full enviroment is passed to scripts with a number of [gw-specific environment variables](/reference/environment-variables). If you want to use variables make sure to use singlequotes so they aren't expanded beforehand.

```sh
gw /path/to/repo -s 'ls -l $BUILD_DIRECTORY | wc -l'
```

Best use-cases for scripts:

- [compile](/guides/compiled) or transpile your code,
- rebuild some assets,
- restart or reload a separately running program.

## Processes

If you have some long-running program, you can use `--process` or `-p` to start as a background process and `gw` will restart it on every pull:

```sh
gw /path/to/repo -p 'ping 1.1.1.1'
```

Processes are started when `gw` is started and they are kept in the background. If there is a change the process is stopped and a new process is started. If you want to look at the output of process, you have to increase verbosity (`-v`):

```sh
$ gw /path/to/repo -v -s 'ping 1.1.1.1'
2024-03-10T15:04:37.740Z INFO  [gw_bin::start] There are updates, running actions.
2024-10-16T18:04:25.888Z INFO  [gw_bin::actions::process] Starting process "ping" in /path/to/repo.
2024-10-16T18:04:25.888Z DEBUG [gw_bin::start] Waiting on triggers.
2024-10-16T18:04:25.888Z INFO  [gw_bin::triggers::schedule] Starting schedule in every 1m.
2024-10-16T18:04:25.906Z DEBUG [gw_bin::actions::process] [ping] PING 1.1.1.1 (1.1.1.1) 56(84) bytes of data.
2024-10-16T18:04:25.906Z DEBUG [gw_bin::actions::process] [ping] 64 bytes from 1.1.1.1: icmp_seq=1 ttl=57 time=16.8 ms
```

Unlike scripts, you can only define one process and they are executed directly instead of in a shell. Processes also can't access gw-specific environment variables. 

If a process fails, by default it marked failed and an error printed. If you want to retry the process you can set the `--process-retries` flag:

```sh
gw /path/to/repo -v -s 'ping 1.1.1.1' --process-retries 5
```

You can also change the stopping behaviour. By default processes are first tried to be gracefully stopped with SIGINT and after some timeout (default: 10s) they are killed. If you want to influence these values you can set `--stop-signal` and `--stop-timeout` respectively. On non-Unix systems these options do nothing and the process is always killed.

```sh
gw /path/to/repo -v -s 'ping 1.1.1.1' --stop-signal SIGTERM --stop-timeout 10s
```

Best use-cases for processes:

- run [interpreted programs](/guides/interpreted) e.g. web frameworks,
- run binaries after [compiling](/guides/compiled),
- run external programs to restart [on config change](/guides/configuration).
