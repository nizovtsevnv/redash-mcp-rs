# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.2.0] - 2026-07-14

### Added
- Windows (x86_64) pre-built binary attached to GitHub releases, packaged as a `.zip`.

### Fixed
- HTTP-mode runtime construction on Windows, where the `rt-multi-thread` tokio feature is intentionally disabled — falls back to a current-thread runtime.
- Flaky HTTP integration tests: the harness now polls the health endpoint until the server is ready instead of waiting a fixed delay.

### Changed
- Documentation: added `--stdio` flag to MCP client configuration examples and refreshed the README to reflect the current project state.

## [2.1.0]

Initial tracked release.

[2.2.0]: https://github.com/nizovtsevnv/redash-mcp-rs/releases/tag/v2.2.0
[2.1.0]: https://github.com/nizovtsevnv/redash-mcp-rs/releases/tag/v2.1.0
