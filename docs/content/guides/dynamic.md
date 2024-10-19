+++
title = "Dynamic languages"
weight = 2
+++

# Dynamic languages

For dynamic languages you can use `gw` very simply with [processes](/usage/actions#processes).

## Configuration

Wrap the way you normally start your program with the `--process` flag.

### Node.js

For example for Node.js programs, use the usual `npm run start` script with a process:

```sh
gw /path/to/repo -p 'npm run start'
```

You can also run the unit tests first, if you want to make sure to restart if the code is in a correct state:

```sh
gw /path/to/repo -s 'npm run test' -p 'npm run start'
```

If you want to use a build step, for example for TypeScript or Next.js, look at the [TypeScript guide](/guides/compiled#typescript).

### Python

Use the same idea with Python, wrap your program's entrypoint in a process:

```sh
gw /path/to/repo -p 'python manage.py runserver'
```

### Ruby

Same thing with Ruby, add process to your program's entrypoint:

```sh
gw /path/to/repo -p 'bin/rails server'
```
