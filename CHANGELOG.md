# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2026-06-03 - Initial release

### Added
- Added the initial RustFS multi-call release.
- Added a command manual in `manual/index.html`.
- Added this changelog for release tracking.

### Validation
- Verified with `cargo build --release`.
- Verified boot with QEMU using the generated initramfs.
- Verified smoke tests for help output, shell redirection, and pipeline file output behavior.
