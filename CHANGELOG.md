# Changelog

## [Unreleased]

### Changed
- Updated test dependencies
- Updated ureq to avoid ring vulnerability
- Updated duct to 1.0.0

## [0.4.1] - 2025-01-26

### Changed

- **New feature**: Trigger on git tags
  - Use the `--on tag` to only pull to the latest tag on the branch
  - Use `--on tag:v*` to match the tags with the given glob
  - Git repository will stay on the branch, but will do a partial pull to the tag and run actions
- Updated dependencies, with libgit2 updated to 1.9.0

## [0.4.0] - 2024-10-23

### Added

- **New feature**: Subprocess handling
  - Use `-p` to start a process directly and restart on change
  - Configure the retries to restart the process in case of a failure
  - Set the stop signal and stop timeout args to configure graceful shutdown before restart
  - Add `-P` to start the process in the shell instead
- The order of script and process flags now matter, scripts are run in order before and after the process
- Add testing for Windows and MacOS machines

### Changed

- **Breaking change**: Scripts are now running directly, you can run it in a shell using `-S`
- Only change gitconfig (safe.directory) if there isn't one
- Don't overwrite script environment, use already set variables
- If the user presses Ctrl+C a second time, the program exits immediately

## [0.3.2] - 2024-08-26

### Added

- Add Docker image support for arm/v7

### Changed

- Make ARM binaries statically linked 32-bit to maintain compatibility with older devices

## [0.3.1] - 2024-08-21

### Added

- Support cross-compilation for Linux ARM machines
- Support compilation for MacOS ARM machines
- Support multi-platform Docker images
- Add Changelog to releases automatically

### Changed

- Fix accidentally dropped Windows support

## [0.3.0] - 2024-08-19

### Added

- Add context to share data between different steps
- Expose the context through environmental variables for the scripts
- Add documentation for the environmental variables
- Add `--version` flag to print current version
- Add `--quiet` flag to improve logging
- Add signal handling to handle SIGINT and SIGTERM
- Add `--ssh-key` flag to change the ssh-key path
- Add `--git-username` and `--git-token` flags to change the https authentication
- Generate `.ssh/known_hosts` file if there is none found on the system
- Add `--git-known-host` to add an entry to the `.ssh/known_hosts` file
- Add installation script

### Changed

- Change musl release to build everything statically
- Ignore different owner repository warnings
- Improve error messages, print original error in fetch

### Removed

- Remove `-o` short flag for `--once`
- Remove debian-based image to streamline usage
- Remove on tag argument for now

## [0.2.2] - 2024-02-17

### Added

- Add `-v` flag to increase verbosity, default log level changed to INFO
- Add check to avoid pulling, when the repository is dirty
- Add more tracing to git repository
- Add safe directory inside Docker image

### Changed

- Fix bug with tag fetching

## [0.2.1] - 2024-02-12

### Added

- Add Docker image
- Add image building to GitHub Actions

### Changed

- Improve documentation

## [0.2.0] - 2024-02-09

### Changed

- Rewrite code to be more modular
- Introduce tests to every part of the codebase
- Add documentation to every module
- Refactor error handling to use thiserror
- Add testing to GitHub Actions
