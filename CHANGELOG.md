# Changelog

## [Unreleased]

- Add context to share data between different steps
- Expose the context through environmental variables for the scripts
- Add documentation for the environmental variables
- Add `--version` flag to print current version
- Add `--quiet` flag to improve logging
- Remove `-o` short flag for `--once`
- Change musl release to build everything statically
- Ignore different owner repository warnings
- Print original error in fetch
- Add signal handling to handle SIGINT and SIGTERM

## [0.2.2] - 2024-02-17

- Add `-v` flag to increase verbosity, default log level changed to INFO
- Add check to avoid pulling, when the repository is dirty
- Fix bug with tag fetching
- Add more tracing to git repository
- Add safe directory inside Docker image

## [0.2.1] - 2024-02-12

- Add Docker image
- Add image building to GitHub Actions
- Improve documentation

## [0.2.0] - 2024-02-09

- Rewrite code to be more modular
- Introduce tests to every part of the codebase
- Add documentation to every module
- Refactor error handling to use thiserror
- Add testing to GitHub Actions
