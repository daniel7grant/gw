+++
title = "Docker Compose"
weight = 4
+++

# Docker Compose

`gw` plays very well with `docker`, especially `docker compose`. You can use `gw` to build the Docker image and then stop and recreate the container. It can be done with `docker`, but it is recommended to do it with `docker compose` as it will handle the whole lifecycle of build, stop and restart.

## Project configuration

Make sure to have a `docker-compose.yaml` file in the root directory. It can start existing containers or build images from the local files.

> **Note**: if you build docker images locally, you save the storage and transfer costs of the docker image repository.

Since it is also a file in your git repository, it basically doubles as an infrastructure-as-a-code. You can modify the `docker compose` setup (e.g. add another dependency, for example cache), commit and have the changes reflected in your infrastructure immediately.

## gw configuration

Just simply point to your repository and run `docker compose up`. It will restart your containers and apply any new changes.

```sh
gw /path/to/repo -s 'docker compose up -d'
```

If you are building your containers, add `--build`:

```sh
gw /path/to/repo -s 'docker compose up -d --build'
```

## Systemd unit

One neat way to use `docker-compose` is to use it together with [systemd units](/usage/systemd). They play very nicely together
because `docker-compose` is hard to be containerized, but this way it can run in the background on the host and update automatically.

To create a systemd unit, you can use the [systemd usage guide](/usage/systemd#usage), but add this to the unit file (e.g. `/etc/systemd/system/gw.service`):

```ini
# /etc/systemd/system/gw.service
[Unit]
Description = Autorestartable docker-compose for application
After = NetworkManager-wait-online.service network.target docker.service
PartOf = docker.service

[Service]
Type = simple
ExecStart = /usr/local/bin/gw /path/to/repo -v -p '/usr/bin/docker compose -f /path/to/repo/docker-compose.yml up --build'

[Install]
WantedBy = default.target
```

This will rebuild your containers in the directory, every time there is a change.

### Template systemd unit

If you have many applications that you want to autorestart with `docker-compose`, it might make sense to use a [template systemd unit](https://fedoramagazine.org/systemd-template-unit-files/).
These are units that have a pattern (`%I`) in their unit files, which you can call multiple times with multiple configurations.

For example if you have a `/path/to/repos/app1` and `/path/to/repos/app2`, you can create a generic systemd unit such as this
(make sure that the filename ends with `@`):

```ini
# /etc/systemd/system/gw@.service
[Unit]
Description = Autorestartable docker-compose for %I
After = NetworkManager-wait-online.service network.target docker.service
PartOf = docker.service

[Service]
Type = simple
WorkingDirectory = /path/to/repos/%I
ExecStart = /usr/local/bin/gw /path/to/repos/%I -vv -p '/usr/bin/docker compose -f /path/to/repos/%I/docker-compose.yml -p %I up --build'

[Install]
WantedBy = default.target
```

You can call it using the app name after a `@`, like `gw@app1` and `gw@app2`, and the `%I` will be automatically replaced with `app1` and `app2`.
So in this case `systemctl start gw@app1` will start a `docker-compose` in `/path/to/repos/app1` using the `/path/to/repos/app1/docker-compose.yml` file
with the project name `app1`.

You can extend these further by simply adding new directories and starting an automaticly deploying process with one line of code.
