# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased] - unreleased

### Added

- ([#38], [#45]) CLI flag for deleting only by age, and ignore count

[#38]: https://github.com/NobbZ/nix-janitor/issues/38
[#45]: https://github.com/NobbZ/nix-janitor/issues/45

### Changed

_nothing_

### Deprecated

_nothing_

### Removed

_nothing_

### Fixed

_nothing_

### Security

_nothing_

## [0.2.0] - 2024-05-20

### Added

- (#32) CLI flags to override the default keep
- (#32) CLI flags to control verbosity

### Changed

_nothing_

### Deprecated

_nothing_

### Removed

_nothing_

### Fixed

_nothing_

### Security

- (#36) replace insecure dependency `users` with `uzers`
- (#36) drop `is-root` in favor of a oneliner implementing the same feature
