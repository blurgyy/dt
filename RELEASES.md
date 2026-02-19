# Release Notes

## 0.8.0

### New Features

- **Reverse Collection (`dt collect`)**: New command to collect files from target directories back to source directories. This enables bidirectional syncing between your configuration repository and live system configurations.
  - Git dirty check with fail-fast behavior to prevent accidental overwrites of uncommitted changes
  - Exclude patterns support for reverse collection
  - Source-target conflict detection
  - Orphan file detection and improved sync logic
  - `collect_sources` for target file discovery

### Improvements

- Updated to **Rust 2024 edition**
- Replaced `pretty_env_logger` with `tracing-subscriber` for improved logging and observability

### Dependencies

- Updated `toml` from 0.7.6 to 0.8.1
- Updated `dirs` from 4.0.0 to 5.0.1
- Updated `path-clean` to 1.0.1
- Updated `tokio` from 1.22.0 to 1.23.1
- Updated `h2` from 0.3.15 to 0.3.17
- Updated `shellexpand` to 3.1.0
- Updated `gethostname` to 0.4.3

### Development

- Added comprehensive tests for collect functionality
- Updated roadmap to reference documentation branch
- Removed design.md from main branch (moved to docs branch)
- Added Nix flake support for reproducible development environments

---

## Previous Releases

See git tags for earlier release history.
