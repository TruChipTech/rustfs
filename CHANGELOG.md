# Changelog

All notable changes to this project will be documented in this file.

## [1.2.0] - 2026-06-28

### Added
- Added 3 new applets: `chroot`, `kexec`, `switch_root`.
- Began a BusyBox parity effort, adding 56 new applets and 3 aliases:
  - **Text/encoding:** `comm`, `cal`, `cksum`, `sum`, `expand`, `unexpand`,
    `split`, `uuencode`, `uudecode`, `unix2dos`, `dc`, `sha1sum`, `sha512sum`,
    `dnsdomainname`.
  - **Aliases:** `egrep` (`grep -E`), `fgrep` (`grep -F`), `zcat` (`gunzip -c`).
  - **Process/scheduling:** `pidof`, `pgrep`, `pkill`, `killall5`, `setsid`,
    `usleep`, `nice`, `renice`, `ionice`, `chrt`, `taskset`, `time`, `watch`.
  - **Session/terminal/mounts:** `who`, `mesg`, `ttysize`, `mountpoint`,
    `pivot_root`.
  - **Disk/device/kernel:** `mknod`, `mkfifo`, `devmem`, `eject`,
    `freeramdisk`, `swapon`, `swapoff`, `sysctl`, `findfs`, `mkswap`, `rdev`,
    `lsattr`, `chattr`, `fdformat`, `hdparm`, `flash_lock`, `flash_unlock`,
    `readprofile`, `rtcwake`, `adjtimex`, `raidautorun`, `fdflush`.
- Each new applet is selectable via `Kconfig` (`CONFIG_APPLET_*`, default `y`).
- Applet count increased from 135 to 197 (138 plus the 56 new applets and 3
  aliases).

### Changed
- `sed`: refactored line processing into a shared `apply_commands` path; added
  multi-command output semantics (auto-print of the pattern space, `p` flag on
  `s///`, and the `q` command stopping processing).
- `last`: replaced the `libc::localtime` based time formatting with `chrono`
  for safe, timezone-aware timestamps.
- Release profile tuned for smaller binaries: `opt-level = "z"`, `lto = "fat"`,
  `codegen-units = 1`, and disabled overflow checks / incremental builds.
- Added the `sha1` crate dependency to back the new `sha1sum` applet.

### Fixed
- `sh`: pipelines now create pipes with `O_CLOEXEC` so raw fds are not leaked
  into unrelated children, and children restore the default `SIGPIPE`
  disposition so producers (e.g. `yes | head`) terminate on a broken pipe
  instead of hanging.

---

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
