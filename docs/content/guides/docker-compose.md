+++
title = "Docker Compose"
weight = 3
+++

# Docker Compose

`gw` plays very well with `docker`, especially `docker compose`. You can use `gw` to build the Docker image and then stop and recreate the container. It can be done with `docker`, but it is recommended to do it with `docker compose` as it will handle the whole lifecycle of build, stop and restart.

## Project configuration

Make sure to have a `docker-compose.yaml` file in the root directory. It can start existing containers or build images from the local files. 

> Note: if you build docker images locally, you save the storage and transfer costs of the docker image repository.

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