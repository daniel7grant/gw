+++
title = "PHP"
weight = 1
+++

# PHP

The simplest configuration is for PHP, because you don't have to build or restart anything.

## Configuration

Just simply set `gw` to watch the directory and it will pull the changes:

```sh
gw /path/to/directory
```

In case, you don't have access in your shared hosting to start long-running tasks, but you can run cronjobs (e.g. CPanel), you can still use `gw`. Just download the binary and add to the crontab with the `--once` flag:

```sh
* * * * * gw /path/to/directory --once
```

This will pull changes every minute.