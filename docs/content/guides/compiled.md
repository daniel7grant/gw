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

If you want to ensure that your code is correct, you can run the unit tests first:

```sh
gw /path/to/repo  -s 'npm run test' -s 'npx tsc -p tsconfig.json' -p 'node dist/index.js'
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

You can add testing as a script, if you want to run the unit tests before the code is deployed:

```sh
gw /path/to/repo -s 'go test' -s 'go build main.go' -p './main'
```

### Rust

For Rust, you can either run it directly or build it first and run it as a subprocess:

```sh
gw /path/to/repo -p 'cargo run --release'
# or
gw /path/to/repo -s 'cargo build --release' -p './target/release/repo'
```

Add the tests here as well to make sure that the code is correct:

```sh
gw /path/to/repo -s 'cargo test' -s 'cargo build --release' -p './target/release/repo'
```

Also checkout [Docker configuration](/guides/docker-compose).
