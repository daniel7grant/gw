+++
title = "Compiled languages"
weight = 3
+++

# Compiled languages

For compiled (Go, Rust) or transpiled (TypeScript) you can use `gw` to build new assets then restart the running binary to restart the server.

## Configuration

Simply add the scripts to build the binary then another one to restart it and watch the repository.

### TypeScript

For example for TypeScript, transpile to JS and run with Node.js:

```sh
gw /path/to/repo -s 'npx tsc -p tsconfig.json' -p 'node dist/index.js'
```

For Next.js and other frameworks that require a build step before starting, you can use:

```sh
gw /path/to/repo -s 'npm run build' -p 'npm run start'
```

### Go

For Go, you can either run it directly or build it first and run it as a subprocess:

```sh
gw /path/to/repo -p 'go run main.go'
# or 
gw /path/to/repo -s 'go build main.go' -p './main'
```

### Rust

For Rust, you can either run it directly or build it first and run it as a subprocess:

```sh
gw /path/to/repo -p 'cargo run --release'
# or 
gw /path/to/repo -s 'cargo build --release' -p './target/release/repo'
```

Also checkout [Docker configuration](/guides/docker-compose).
