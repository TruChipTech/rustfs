# RustFS — Rust-based Rootfs Utilities

A safe, correct, and fast multi-call binary implementing common Unix utilities in Rust. Provides essential rootfs userland tools in a single, statically-linkable binary.

## Key Advantages

- **Memory Safety**: Written in Rust — no buffer overflows, use-after-free, or integer overflow vulnerabilities
- **No Race Conditions**: Proper file handling eliminates TOCTOU bugs
- **Correct Symlink Handling**: Prevents symlink-following attacks during recursive operations
- **Safe Integer Arithmetic**: Checked arithmetic prevents silent overflow
- **Proper UTF-8 Handling**: Gracefully handles invalid encodings instead of crashing
- **Cross-platform**: Works on Linux, macOS, and Windows

## Included Applets (190+)

### Init System (mutually exclusive — one at a time)
`init` — Linux init program (PID 1), selectable at build time:
- **RustFS init** — lightweight, simple init (default)
- **System V init** — full SysV with runlevels and /etc/inittab
- **systemd-compatible init** — parses .service unit files

### Device Manager
`mdev` — lightweight device manager (hotplug, /etc/mdev.conf)
`udevd` `udevadm` — udev-compatible device manager (/etc/udev/rules.d/)

### File Operations
`cat` `cp` `mv` `rm` `mkdir` `rmdir` `ls` `touch` `ln` `chmod` `chgrp` `chown` `head` `tail` `tee` `wc` `du` `df` `dd` `stat` `readlink` `install` `less`

### Text Processing
`echo` `printf` `grep` `sed` `sort` `uniq` `tr` `cut` `paste` `fold` `rev` `nl` `awk` `diff` `dos2unix` `getopt` `hd` `hexdump` `length`

### Path / String
`basename` `dirname` `pwd` `realpath`

### Encoding / Hashing
`base64` `md5sum` `sha256sum` `xxd`

### System Info
`uname` `hostname` `whoami` `id` `uptime` `date` `env` `printenv` `hostid` `logname`

### Process / Misc
`sleep` `yes` `true` `false` `nohup` `seq` `tty` `which` `xargs` `find` `test` `expr` `fuser` `getty` `kill` `killall` `login` `last`

### User / Group Management
`addgroup` `adduser` `delgroup` `deluser`

### Networking
`arp` `arping` `ftpd` `ftpget` `ftpput` `httpd` `ifconfig` `ifdown` `ifup` `ip` `ipaddr` `ipcalc`

### Archive / Compression
`ar` `bunzip2` `bzcat` `bzip2` `gunzip` `gzip`

### Disk / System Utilities
`blkid` `clear` `dmesg` `fbset` `fdisk` `fsck` `fsync` `hwclock` `insmod` `klogd` `losetup` `lsmod`

### Boot / Kernel
`chroot` — run a command (or shell) with a different root directory
`kexec` — load and boot into a new kernel without a firmware reboot (via `kexec_file_load(2)`)
`switch_root` — free the initramfs and switch to the real root filesystem (the final step of an initramfs)

### IPC Utilities
`ipcrm` `ipcs`

### Logging
`logger` `logread`

### Extended Applet Additions (v1.2.0)
Text / encoding: `comm` `cal` `cksum` `sum` `expand` `unexpand` `split` `uuencode` `uudecode` `unix2dos` `dc` `sha1sum` `sha512sum` `dnsdomainname`
Aliases: `egrep` (`grep -E`) `fgrep` (`grep -F`) `zcat` (`gunzip -c`)
Process / scheduling: `pidof` `pgrep` `pkill` `killall5` `setsid` `usleep` `nice` `renice` `ionice` `chrt` `taskset` `time` `watch`
Session / terminal / mounts: `who` `mesg` `ttysize` `mountpoint` `pivot_root`
Disk / device / kernel: `mknod` `mkfifo` `devmem` `eject` `freeramdisk` `swapon` `swapoff` `sysctl` `findfs` `mkswap` `rdev` `lsattr` `chattr` `fdformat` `hdparm` `flash_lock` `flash_unlock` `readprofile` `rtcwake` `adjtimex` `raidautorun` `fdflush`

> Note: additional applets (networking tools, archivers, daemons, and
> editors) are being added in later phases of the parity effort.

## Configuration (Kconfig)

RustFS uses a Kconfig-based build configuration system. Each applet and init subsystem can be individually enabled or disabled.

```bash
# Load default config (all features enabled)
./configure.sh defconfig

# Generate a complete config straight from Kconfig (every applet/feature;
# cannot drift behind newly-added applets)
./configure.sh allyesconfig

# Load minimal config (init + essential applets only)
./configure.sh minimal

# Interactive menuconfig (requires python3 + kconfiglib)
./configure.sh menuconfig

# Show current configuration
./configure.sh show
```

The configuration is stored in `.config`. You can also edit it manually — it uses the standard `CONFIG_*=y` format.

### Key Config Options

| Option | Description |
|--------|-------------|
| `CONFIG_INIT` | Enable init applet (PID 1 support) |
| `CONFIG_INIT_RUSTFS` | RustFS simple init (lightweight, default) |
| `CONFIG_INIT_SYSVINIT` | System V init (/etc/inittab, runlevels) |
| `CONFIG_INIT_SYSTEMD` | systemd-compatible init (.service files) |
| `CONFIG_MDEV` | mdev lightweight device manager |
| `CONFIG_UDEV` | udev-compatible device manager |
| `CONFIG_APPLET_*` | Individual applet enable/disable |

**Note:** Only one init system can be enabled at a time (enforced by Kconfig `choice`).

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

The binary is at `target/release/rustfs` (~1.5 MB stripped).

### Optimizing for Size

The release profile is tuned to produce the smallest practical binary — important
when RustFS is the entire userland of an initramfs or embedded rootfs:

| Setting | Effect |
|---------|--------|
| `opt-level = "z"` | Optimize aggressively for size over speed |
| `lto = "fat"` | Whole-program link-time optimization across all crates |
| `codegen-units = 1` | Single codegen unit for maximum cross-module size optimization |
| `panic = "abort"` | Drops unwinding tables and landing pads |
| `strip = true` | Strips symbols and debug info |
| `overflow-checks = false` | Removes overflow-check branches in release |

To shrink the binary further, disable applets you don't need via Kconfig
(`./configure.sh menuconfig` or by editing `.config`) — each disabled applet is
compiled out entirely. For the smallest possible image, start from the minimal
config:

```bash
./configure.sh minimal
cargo build --release
```

For fully static, dependency-free binaries (ideal for initramfs), build against
musl with the bundled libbz2:

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl --features static-bin
```

## Cleaning Build Artifacts

```bash
# Clean build artifacts
cargo clean

# Clean install directory
./install.sh clean

# Full clean (build artifacts + .config + install directory + rootfs.cpio.gz)
./install.sh distclean
# or equivalently:
./configure.sh distclean
```

## Usage

For command-by-command details, option meanings, and examples, see the HTML manual in [manual/index.html](manual/index.html).
It is organized as a multi-call command reference and groups commands by category.

### Subcommand Mode
```bash
rustfs cat /etc/passwd
rustfs grep -r "TODO" src/
rustfs find . -name "*.rs" -type f
```

### Symlink Mode
```bash
# Create symlinks
ln -s rustfs /usr/local/bin/cat
ln -s rustfs /usr/local/bin/grep
ln -s rustfs /usr/local/bin/ls

# Then use directly
cat /etc/passwd
grep -r "TODO" src/
ls -la
```

## Installing

```bash
# Build and install rootfs to _install/ (default)
cargo build --release
sudo ./install.sh

# Install to custom directory
sudo ./install.sh INSTALL_DIR=/opt/rustfs

# Clean
./install.sh clean
```

The installer creates a complete minimal rootfs with:
- User-facing utilities in `INSTALL_DIR/bin/`
- System administration tools in `INSTALL_DIR/sbin/`
- `/init` symlink at root for kernel boot
- All necessary config files, init scripts, and device manager setup based on `.config`

## Using as Linux Init (PID 1)

RustFS can serve as the init process for a Linux system. Symlink or copy the binary as `/sbin/init`:

```bash
ln -sf /path/to/rustfs /sbin/init
```

### RustFS Init (default, CONFIG_INIT_RUSTFS)

Lightweight init similar to minimal init implementations. Reads a simplified `/etc/inittab`, mounts essential filesystems, runs rc scripts, and respawns processes.

```bash
# Kernel boot parameter
init=/sbin/init
```

Example `/etc/inittab`:
```
# System initialization
rc::sysinit:/etc/init.d/rcS

# Start getty on console
tty1::respawn:/sbin/getty 38400 tty1
tty2::respawn:/sbin/getty 38400 tty2

# Ask before launching shell
console::askfirst:/bin/sh

# Ctrl-Alt-Del
ca::ctrlaltdel:/sbin/reboot
```

Supported actions: `sysinit`, `wait`, `once`, `respawn`, `askfirst`, `shutdown`, `restart`, `ctrlaltdel`

### System V Init (CONFIG_INIT_SYSVINIT)

Full System V init with runlevel support. Parses standard `/etc/inittab`:

```
# Default runlevel
id:3:initdefault:

# System initialization
si::sysinit:/etc/init.d/rcS

# Runlevel scripts
l3:3:wait:/etc/init.d/rc 3

# Respawn getty on ttys
1:2345:respawn:/sbin/getty 38400 tty1
2:2345:respawn:/sbin/getty 38400 tty2

# Ctrl-Alt-Del
ca::ctrlaltdel:/sbin/reboot
```

### systemd-Compatible Init (CONFIG_INIT_SYSTEMD)

Parses `.service` unit files from standard systemd paths:

Supported `.service` directives:
- **[Unit]**: `Description`, `After`, `Before`, `Requires`, `Wants`
- **[Service]**: `Type` (simple/forking/oneshot), `ExecStart`, `ExecStartPre`, `ExecStartPost`, `ExecStop`, `Restart` (always/on-failure/no), `RestartSec`, `WorkingDirectory`, `Environment`
- **[Install]**: `WantedBy`, `RequiredBy`

## Device Manager

### mdev (CONFIG_MDEV)

Lightweight device manager for populating `/dev`:

```bash
# Scan /sys and populate /dev
mdev -s

# Run as daemon (listen for hotplug)
mdev -d

# Called by kernel as hotplug handler
echo /sbin/mdev > /proc/sys/kernel/hotplug
```

Rules in `/etc/mdev.conf`:
```
# <pattern> <uid>:<gid> <mode> [<action> <param>]
tty[0-9]*     0:5   0660
sd[a-z]*      0:6   0660  * /etc/mdev/storage.sh
null          0:0   0666
zero          0:0   0666
random        0:0   0444
urandom       0:0   0444
console       0:0   0600
```

### udev (CONFIG_UDEV)

udev-compatible device manager:

```bash
# Run as daemon
udevd

# Trigger coldplug for existing devices
udevadm trigger

# Show device info
udevadm info /dev/sda
```

Reads rules from `/etc/udev/rules.d/` and `/usr/lib/udev/rules.d/`. Supported keys:
- **Match**: `SUBSYSTEM`, `KERNEL`, `ACTION`, `DEVPATH`, `ATTR{...}`, `ENV{...}`
- **Assign**: `NAME`, `SYMLINK`, `MODE`, `OWNER`, `GROUP`, `RUN`, `ENV{...}`

## Creating a Minimal Rootfs

`install.sh` generates a complete minimal rootfs with all necessary init scripts, device manager configuration, directory structure, and device nodes based on your `.config`.

```bash
# Build first
cargo build --release

# Install rootfs (reads .config to determine init/devmgr)
sudo ./install.sh

# Install to custom path
sudo ./install.sh INSTALL_DIR=/tmp/myrootfs

# Clean install directory
./install.sh clean
```

The script automatically configures:
- **RustFS init** → `/etc/inittab` (simple format) + `/etc/init.d/rcS`
- **System V init** → `/etc/inittab` (SysV format) + runlevel dirs + `/etc/init.d/rc`
- **systemd init** → `.service` unit files + target structure
- **mdev** → `/etc/mdev.conf` with device rules
- **udev** → `/etc/udev/rules.d/` with default rules

### Testing on a Regular Linux System (chroot)

```bash
# Build and create rootfs
cargo build --release
sudo ./install.sh

# Test with chroot (does not run init, just tests applets)
sudo chroot _install /bin/sh

# Inside chroot:
ls /
cat /etc/hostname
echo "Hello from RustFS"
exit
```

### Default Login Credentials

| User | Password |
|------|----------|
| `root` | *(no password)* |

The rootfs is configured with passwordless root login for testing. To set a root password, edit `_install/etc/shadow` and replace the empty field with a crypt hash, or boot and run `passwd` if available.

### QEMU testing prerequisites

> **Enable all applets first.** `./configure.sh defconfig` loads
> `configs/default_defconfig`, which now enables every applet defined in
> `Kconfig`. If you build without it (or with a stale `.config`), disabled
> applets are *compiled out* and report `unknown applet` at runtime even though
> they appear in `--help`.
>
> ```bash
> ./configure.sh defconfig      # enable all features before building
> ```
>
> **The rootfs needs applet symlinks + PATH.** RustFS is a multi-call binary and
> its bundled `sh` has no shell builtins — it `exec`s external commands. The
> rootfs must contain `/bin/rustfs`, a `/bin/<applet>` symlink per applet
> (including `/bin/sh`), and `/init` must `export PATH=/bin`. `install.sh`
> creates these symlinks.
>
> **Ready-to-boot kernels** (no need to build one) can be pulled from Debian's
> netboot images — these are plain `bzImage`/`Image`/`zImage` files that boot a
> custom initramfs directly:
>
> ```bash
> base=http://deb.debian.org/debian/dists/stable/main
> curl -o vmlinuz-amd64 "$base/installer-amd64/current/images/netboot/debian-installer/amd64/linux"
> curl -o Image-arm64   "$base/installer-arm64/current/images/netboot/debian-installer/arm64/linux"
> curl -o vmlinuz-armhf "$base/installer-armhf/current/images/netboot/vmlinuz"
> curl -o Image-riscv64 "$base/installer-riscv64/current/images/netboot/debian-installer/riscv64/linux"
> ```
>
> **Automated harness.** `qemu-test/run.sh <x86_64|arm64|arm32|riscv64>` builds
> the target, packages a RustFS-only initramfs, boots it under `qemu-system-*`
> (full-system, not qemu-user), runs the in-guest applet tests and scores them.
> Results are written to `qemu-test/work/results-<arch>.txt`.
>
> **Full build matrix.** `qemu-test/run_matrix.sh [arch...]` boots every
> combination of **{release, debug} × {static, shared}** for each architecture
> under `qemu-system-*` and writes `qemu-test/work/MATRIX.txt`. Static images use
> musl (x86_64/arm64/arm32) or glibc + `crt-static` (riscv64); shared images are
> dynamically linked and the binary's glibc dependency closure (loader + libc +
> libm + libgcc_s + libcrypt) is bundled into the initramfs automatically. Filter
> with `RUN_LINKS=shared RUN_PROFILES=release qemu-test/run_matrix.sh arm64`.
> Latest result — all four architectures, all four build modes:
>
> | Arch | release/static | debug/static | release/shared | debug/shared |
> |------|:--:|:--:|:--:|:--:|
> | x86_64  | 135/135 | 135/135 | 135/135 | 135/135 |
> | arm64   | 135/135 | 135/135 | 135/135 | 135/135 |
> | arm32   | 135/135 | 135/135 | 135/135 | 135/135 |
> | riscv64 | 134/135 | 134/135 | 134/135 | 134/135 |
>
> (riscv64: the single non-pass is `devmem 0x0` reading an unmapped physical
> address — an architecture/robustness edge case, not a build or link failure.)

### Testing with QEMU (x86_64)

```bash
# 0. Enable all applets
./configure.sh defconfig

# 1. Build and install rootfs
cargo build --release
sudo ./install.sh

# 2. Create initramfs
cd _install && find . | cpio -o -H newc | gzip > ../rootfs.cpio.gz && cd ..

# 3. Get a kernel (use your distro's kernel or build one)
# Option A: Use host kernel
sudo cp /boot/vmlinuz-$(uname -r) ./vmlinuz && sudo chmod 644 ./vmlinuz
KERNEL=./vmlinuz

# Option B: Download a minimal kernel
# wget https://kernel.org/.../linux-x.x.tar.xz && make defconfig && make -j$(nproc)

# 4. Boot with QEMU
qemu-system-x86_64 \
    -kernel "$KERNEL" \
    -initrd rootfs.cpio.gz \
    -append "console=ttyS0 init=/init panic=1" \
    -nographic \
    -m 256M \
    -no-reboot

# To exit QEMU: Ctrl-A then X
```

### Testing with QEMU (ARM - 32-bit)

```bash
# 1. Cross-compile for ARM
rustup target add armv7-unknown-linux-musleabihf
cargo build --release --target armv7-unknown-linux-musleabihf

# 2. Create rootfs with the ARM binary
sudo BINARY=target/armv7-unknown-linux-musleabihf/release/rustfs ./install.sh

# 3. Create initramfs
cd _install && find . | cpio -o -H newc | gzip > ../rootfs.cpio.gz && cd ..

# 4. Get an ARM kernel (vexpress or versatile)
# Download prebuilt: https://mirrors.edge.kernel.org/pub/linux/kernel/

# 5. Boot with QEMU
qemu-system-arm \
    -M virt \
    -cpu cortex-a15 \
    -kernel zImage \
    -initrd rootfs.cpio.gz \
    -append "console=ttyAMA0 init=/init panic=1" \
    -nographic \
    -m 256M \
    -no-reboot

# To exit QEMU: Ctrl-A then X
```

### Testing with QEMU (ARM64 / AArch64)

```bash
# 1. Cross-compile for ARM64
rustup target add aarch64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl

# 2. Create rootfs with the ARM64 binary
sudo BINARY=target/aarch64-unknown-linux-musl/release/rustfs ./install.sh

# 3. Create initramfs
cd _install && find . | cpio -o -H newc | gzip > ../rootfs.cpio.gz && cd ..

# 4. Get an ARM64 kernel
# Build or download a prebuilt Image for aarch64

# 5. Boot with QEMU
qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a57 \
    -kernel Image \
    -initrd rootfs.cpio.gz \
    -append "console=ttyAMA0 init=/init panic=1" \
    -nographic \
    -m 512M \
    -no-reboot

# To exit QEMU: Ctrl-A then X
```

### Testing with QEMU (RISC-V 64)

RISC-V needs a cross C toolchain (for the bundled `bzip2` C code) and `libcrypt`.
On Debian/Ubuntu hosts: `sudo apt install gcc-riscv64-linux-gnu`. If you cannot
install system packages, the debs can be extracted into a local sysroot (see
`qemu-test/run.sh` for the exact extraction used here).

```bash
# 1. Cross-compile (static; +crt-static so no riscv loader is needed in the rootfs)
rustup target add riscv64gc-unknown-linux-gnu
export CC_riscv64gc_unknown_linux_gnu=riscv64-linux-gnu-gcc
export CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_LINKER=riscv64-linux-gnu-gcc
export CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-static"
cargo build --release --target riscv64gc-unknown-linux-gnu

# 2. Rootfs + initramfs (as above, with the riscv64 binary)
sudo BINARY=target/riscv64gc-unknown-linux-gnu/release/rustfs ./install.sh
cd _install && find . | cpio -o -H newc | gzip > ../rootfs.cpio.gz && cd ..

# 3. Boot with QEMU (virt machine uses the built-in OpenSBI firmware)
qemu-system-riscv64 \
    -M virt \
    -kernel Image-riscv64 \
    -initrd rootfs.cpio.gz \
    -append "console=ttyS0 init=/init panic=1" \
    -nographic -m 512M -no-reboot

# To exit QEMU: Ctrl-A then X
```

### Quick Test Script

For convenience, a full build-and-test cycle:

```bash
#!/bin/sh
# test-qemu.sh — Build, create rootfs, boot in QEMU
set -e

cargo build --release
sudo ./install.sh
cd _install && find . | cpio -o -H newc | gzip > ../rootfs.cpio.gz && cd ..

KERNEL="${KERNEL:-/boot/vmlinuz-$(uname -r)}"
qemu-system-x86_64 \
    -kernel "$KERNEL" \
    -initrd rootfs.cpio.gz \
    -append "console=ttyS0 init=/init panic=1" \
    -nographic -m 256M -no-reboot
```

## Safety Guarantees

| Scenario | RustFS Behavior |
|----------|----------------|
| `cp -r` copies directory into itself | Detects and prevents self-copy |
| `mv` same file to itself | Detects and reports error |
| `sed -i` write failure | Atomic temp file + rename preserves original |
| `tail -f` file truncation | Resets position on truncate |
| `rm -rf /` without `--no-preserve-root` | Refuses to remove root |
| Race conditions in recursive operations | Rust's safe file APIs prevent TOCTOU |
| Invalid UTF-8 input | Graceful error handling |
| Integer overflow in `expr` | Checked arithmetic with error |
| `sort` ordering of equal elements | Always uses stable sort |

## License

MIT
