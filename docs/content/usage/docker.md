+++
title = "Docker container"
weight = 5
+++

# Docker container

If you don't want to install or run `gw` on your server, you can also use the prebuilt Docker images at [danielgrant/gw](https://hub.docker.com/r/danielgrant/gw).

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
      - type: volume
        source: ~/.ssh
        target: /root/.ssh
        read_only: true
```

If you are using ssh-keys, mount the `.ssh` directory as well, so it can pull. For more information, see [Authentication](/reference/authentication).

## Customization

### Copy binary from gw

Most applications have many dependencies and complicated setups, and are already running on Docker. In these cases it is often preferable to build the `gw` image on top of the already existing application image.

> **Note**: This doesn't mean that these should be running in the same container, but they can use the same base image in two separate containers. It is a common wisdom that one container should run one thing.

For this we can start off of our application image as a base layer and add the `gw` binary in a `COPY` layer. You can simply wrap your existing command using subprocess mode (`-p`) and it will restart the script every time a pull happened.

```dockerfile
FROM example.org/registry/node-image:ubuntu

# Copy from the `gw` image
COPY --from=danielgrant/gw:0.4.2 /usr/bin/gw /usr/bin/gw

ENTRYPOINT ["/usr/bin/gw"]
CMD ["/app", "-p", "npm start"]
```
