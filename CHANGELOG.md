# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial release of PBS Exporter
- Comprehensive Prometheus metrics for PBS 4.x
  - Host system metrics (CPU, memory, disk, load)
  - Datastore capacity metrics
  - Backup snapshot metrics
- PBS API client with authentication support
- HTTP server with `/metrics`, `/health`, and `/` endpoints
- Configuration via TOML files and environment variables
- CLI with `--config` flag
- Structured logging with tracing
- Docker support with multi-stage builds
- Comprehensive documentation and examples
