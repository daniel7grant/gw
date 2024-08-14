+++
title = "Docker container"
weight = 4
+++

# Docker container

If you don't want to install or run `gw` on your server, you can also use the prebuilt Docker images at [danielgrant/gw](https://hub.docker.com/r/danielgrant/gw).

## Variants

The `gw` Docker images come in two flavours: the default image is based on Debian, while the Alpine tag is a smaller, more secure variant. For most use-cases the Debian-based is a safer option, but if you know what you are doing, the Alpine image can be useful.

```sh
# Pull Debian-based
docker pull danielgrant/gw
docker pull danielgrant/gw:0.2.2

# Pull Alpine-based
docker pull danielgrant/gw:alpine
docker pull danielgrant/gw:0.2.2-alpine
```

## Usage

If you just want to pull a repository or run simple scripts, you can run the container with [docker](https://docs.docker.com/engine/install/). You can mount a repository to a directory and watch it. For example:

```sh
docker run -d --name gw -v /path/to/repo:/app danielgrant/gw /app
```

You can also run scripts, but these images are very small and only have a few programs set up:

```sh
docker run -d --name gw -v /path/to/repo:/app danielgrant/gw /app -s "cp -r build/ html/"
```

If you prefer to use `docker-compose`, you can copy this file to a `docker-compose.yaml` and run `docker compose up -d`:

```yaml
# docker-compose.yaml
version: "3"

services:
    gw:
        container_name: gw
        image: danielgrant/gw
        command: /app
        volumes:
            - type: volume
              source: /path/to/repo
              target: /app
```

## Customization

### Build on top of gw

The default image is useful for some cases, but most of the time we need more some other programs for our scripts. In this case it can work to build an image on top of the `gw` docker image. Copy this into a `Dockerfile` and specify `gw` as a base image:

```dockerfile
# Dockerfile
# It is recommended to specify the version
FROM danielgrant/gw:0.2.2

# Install dependencies
RUN apt-get update && \
    apt-get install -y python3

# Don't add `gw` here
CMD ["/app", "-s", "python update.py"]
```

You can build this image into `gw-python` and run it similarly as before:

```sh
docker build -t gw-python .
docker run -d --name gw-python -v /path/to/repo:/app gw-python
```

If you prefer, you can also build this as part of a `docker-compose`, with running `docker compose up -d --build`:

```yaml
# docker-compose.yaml
version: "3"

services:
    gw:
        container_name: gw-python
        build: .
        command: /app
        volumes:
            - type: volume
              source: /path/to/repo
              target: /app
```

### Copy binary from gw

It is useful to install the dependencies on top of `gw`, but most applications have many dependencies and complicated setups, and are already running on Docker. In these cases it is often preferable to build the `gw` image on top of the already existing application image.

**NOTE**: This doesn't mean that these should be running in the same container, but they can use the same base image in two separate containers. It is a common wisdom that one container should run one thing.

For this we can start off of our application image as a base layer and add the `gw` binary in a `COPY` layer. Also note that `gw` needs `ca-certificates`, `openssl` and `openssh-client` installed, so it can interact with the remote server. If your base image doesn't have these, you should install them as well:

```dockerfile
FROM example.org/registry/node-image:ubuntu

# Copy from the `gw` image
COPY --from=danielgrant/gw:0.2.2 /usr/bin/gw /usr/bin/gw

ENTRYPOINT ["/usr/bin/gw"]
CMD ["/app", "-s", "npm run build"]
```

It is similar with `alpine`, but only the `ca-certificates` and `openssh-client` is required:

```dockerfile
FROM example.org/registry/node-image:alpine

# Copy from the `gw` image
COPY --from=danielgrant/gw:0.2.2-alpine /usr/bin/gw /usr/bin/gw

ENTRYPOINT ["/usr/bin/gw"]
CMD ["/app", "-s", "npm run build"]
```

You can build and use it similarly to the previous step.
