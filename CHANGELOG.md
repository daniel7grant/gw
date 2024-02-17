# Changelog

## [Unreleased]

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
