+++
title = "Compiled languages"
weight = 2
+++

# Compiled languages

For compiled (Go, Rust) or transpiled (TypeScript) you can use `gw` to build new assets then restart the running binary to restart the server.

## Configuration

Simply add the scripts to build the binary then another one to restart it and watch the repository.

For example for TS and PM2:

```sh
gw /path/to/repo -s 'npm run build' -s 'npx pm2 restart'
```

Also checkout [Docker configuration](/guides/docker-compose).
