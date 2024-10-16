+++
title = "Crontab"
weight = 6
+++

# Crontab

If you don't have shell access to the server, you can still run `gw` with the crontab.

> **Note:** this will disable some advanced functions like [webhooks](/usage/webhook). Only use this if you cannot use any other solution.

## Usage

There is a `--once` flag in `gw`, that checks the repository for updates and then exits. You can use this to pair with your own scheduled runner to pull for changes manually. Simply open your crontab:

```sh
crontab -e
```

...and add a new line with your `gw` script. You can use `* * * * *` to run it every minute, but you can use more advanced patterns as well (see [crontab.guru](https://crontab.guru/)). For the command, make sure to specify `--once` to avoid running continuously and add `--quiet` so it will only print on a failure:

```sh
* * * * * gw /path/to/repo --once --quiet
```

> **Warning**: Cronjobs are known to be error-prone and hard to debug, so make sure to test this solution extensively before relying on this in the real world.
