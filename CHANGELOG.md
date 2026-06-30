# Changelog

All notable changes to this project will be documented in this file.

## [1.3.0] - 2026-07-01

### Added
- Extended applet set, phase 1 — 15 new applets (+ `linux32`/`linux64` aliases):
  - **Console/VT:** `chvt`, `deallocvt`, `kbd_mode`, `reset`, `beep`.
  - **Exec domain:** `setarch`, `linux32`, `linux64`.
  - **Text/pager:** `catv`, `more`.
  - **Misc:** `run-parts`, `runlevel`, `pipe_progress`, `volname`, `rdate`,
    `mkpasswd`, `cryptpw`.
- `./configure.sh allyesconfig` — generate a complete config directly from
  Kconfig (every applet/feature), so it cannot drift behind newly-added applets.
- `scripts/add_applet.sh` — wires a new applet into all integration points
  (mod.rs, dispatch, help, Cargo check-cfg, Kconfig, defconfig, build.rs).
- QEMU full-system test harness: `qemu-test/run.sh` and `qemu-test/run_matrix.sh`
  boot a rustfs-only initramfs across x86_64/arm64/arm32/riscv64 and the full
  `{release,debug} × {static,shared}` build matrix.

### Fixed
- `sh`: fixed a file-descriptor double-close in pipeline setup (the previous-pipe
  read end was both moved into the child's `Stdio` and explicitly closed),
  which aborted on debug builds and could corrupt pipelines.
- VT/console applets: use portable `ioctl` request typing so musl targets build.
- `kexec`: define the `kexec_file_load` syscall number for riscv64.
- `configs/default_defconfig`: synced to all Kconfig applets so `defconfig`
  enables every applet as documented.

### Removed
- Dropped third-party project name references from docs/comments in favor of
  neutral wording.

## [1.2.0] - 2026-06-28

### Added
- Added 3 new applets: `chroot`, `kexec`, `switch_root`.
- Began an extended applet effort, adding 56 new applets and 3 aliases:
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
