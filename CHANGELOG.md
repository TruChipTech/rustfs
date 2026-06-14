# Changelog

All notable changes to this project will be documented in this file.

## [1.1.0] - 2026-06-15

### Added
- Added 11 new applets: `ps`, `free`, `sync`, `mktemp`, `nproc`, `tac`, `timeout`,
  `od`, `truncate`, `strings`, `cmp`.
- Applet count increased from 124 to 135.

### Changed
- `grep`: added `-H`, `-h`, `-o`, `-x`, `-f FILE`, `-E`, `-P` flags.
- `sed`: added `-f FILE`, `-r`/`-E` flags, `a TEXT` and `q` commands.
- `tail`: added `-F` (follow by name with log-rotation support).

### Fixed
- `gunzip`: now correctly decompresses standard `.gz` files using `flate2`.
- `ipcrm`: implemented `--all=shm|msg|sem|all` flag.

---

## [1.0.0] - 2026-06-03 - Initial release

### Added
- Added the initial RustFS multi-call release.
- Added a command manual in `manual/index.html`.
- Added this changelog for release tracking.

### Validation
- Verified with `cargo build --release`.
- Verified boot with QEMU using the generated initramfs.
- Verified smoke tests for help output, shell redirection, and pipeline file output behavior.
