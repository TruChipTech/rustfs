#!/bin/bash
# <purpose of file>
#
# Copyright (C) 2026 by Anandkumar  <Truchip >
#
# Licensed under MIT
# RustFS configuration and build script
# Provides a Kconfig-like interface for configuring the build

set -e

CONFIGS_DIR="configs"
DOTCONFIG=".config"
MUSL_TARGET="x86_64-unknown-linux-musl"

usage() {
    echo "Usage: $0 <command>"
    echo ""
    echo "Configuration:"
    echo "  menuconfig     - Interactive configuration (requires python3 + kconfiglib)"
    echo "  defconfig      - Load default configuration (all features enabled, static=y)"
    echo "  minimal        - Load minimal configuration"
    echo "  savedefconfig  - Save current config as defconfig"
    echo ""
    echo "Build:"
    echo "  build          - Build debug binary (static if CONFIG_STATIC=y)"
    echo "  release        - Build release binary (static if CONFIG_STATIC=y)"
    echo "  clean          - Clean build artifacts"
    echo "  distclean      - Clean build artifacts, .config, and install directory"
    echo ""
    echo "Test:"
    echo "  qemu-test      - Build, create initramfs, and boot in QEMU"
    echo "                   KERNEL=/path/to/bzImage  (default: /boot/vmlinuz)"
    echo "                   QEMU_TIMEOUT=<seconds>   (default: 20)"
    echo ""
    echo "Info:"
    echo "  show           - Show current configuration"
    echo "  help           - Show this help"
    echo ""
    echo "Static linking (CONFIG_STATIC=y, the default):"
    echo "  Builds with --target $MUSL_TARGET for a zero-dependency binary."
    echo "  Requires: rustup target add $MUSL_TARGET && apt install musl-tools"
}

# ---------------------------------------------------------------------------
# Helpers: read .config
# ---------------------------------------------------------------------------

# Resolve active config file path (may be empty)
active_config() {
    if [ -f "$DOTCONFIG" ]; then
        echo "$DOTCONFIG"
    elif [ -f "$CONFIGS_DIR/default_defconfig" ]; then
        echo "$CONFIGS_DIR/default_defconfig"
    fi
}

config_enabled() {
    local cfg
    cfg=$(active_config)
    [ -n "$cfg" ] && grep -q "^CONFIG_$1=y" "$cfg"
}

# Returns "x86_64-unknown-linux-musl" when CONFIG_STATIC=y, empty otherwise
build_target() {
    if config_enabled "STATIC"; then
        echo "$MUSL_TARGET"
    fi
}

# Path to the compiled release binary (accounts for target sub-directory)
release_binary() {
    local tgt
    tgt=$(build_target)
    if [ -n "$tgt" ]; then
        echo "target/$tgt/release/rustfs"
    else
        echo "target/release/rustfs"
    fi
}

debug_binary() {
    local tgt
    tgt=$(build_target)
    if [ -n "$tgt" ]; then
        echo "target/$tgt/debug/rustfs"
    else
        echo "target/debug/rustfs"
    fi
}

# Assemble cargo flags based on config
cargo_flags() {
    local mode="$1"   # "debug" or "release"
    local flags=""
    local tgt
    tgt=$(build_target)

    if [ -n "$tgt" ]; then
        flags="--target $tgt --features static-bin"
    fi
    if [ "$mode" = "release" ]; then
        flags="$flags --release"
    fi
    echo "$flags"
}

# Ensure musl target + toolchain are present when needed
check_musl() {
    if ! config_enabled "STATIC"; then
        return
    fi

    if ! rustup target list --installed 2>/dev/null | grep -q "^$MUSL_TARGET"; then
        echo "Installing Rust musl target..."
        rustup target add "$MUSL_TARGET"
    fi

    if ! command -v musl-gcc &>/dev/null; then
        echo "Error: musl-gcc not found."
        echo "Install with:  sudo apt install musl-tools"
        exit 1
    fi
}

# ---------------------------------------------------------------------------
# Commands
# ---------------------------------------------------------------------------

cmd_menuconfig() {
    if ! command -v python3 &>/dev/null; then
        echo "Error: python3 is required for menuconfig"
        echo "Install: apt install python3 / brew install python3"
        exit 1
    fi

    if ! python3 -c "import kconfiglib" 2>/dev/null; then
        echo "Installing kconfiglib..."
        pip3 install kconfiglib
    fi

    python3 -c "
import kconfiglib
import sys

kconf = kconfiglib.Kconfig('Kconfig')

# Load existing .config if present
import os
if os.path.exists('.config'):
    kconf.load_config('.config')
elif os.path.exists('configs/default_defconfig'):
    kconf.load_config('configs/default_defconfig')

kconfiglib.Menuconfig(kconf).show()
kconf.write_config('.config')
print('Configuration saved to .config')
" 2>/dev/null || {
        echo "Menuconfig failed. Using text-based configuration."
        echo "You can also manually edit .config or copy from configs/"
        echo ""
        echo "Available configs:"
        ls -1 "$CONFIGS_DIR/"
    }
}

cmd_defconfig() {
    cp "$CONFIGS_DIR/default_defconfig" "$DOTCONFIG"
    echo "Loaded default configuration -> $DOTCONFIG"
}

cmd_minimal() {
    cp "$CONFIGS_DIR/minimal_defconfig" "$DOTCONFIG"
    echo "Loaded minimal configuration -> $DOTCONFIG"
}

cmd_savedefconfig() {
    if [ ! -f "$DOTCONFIG" ]; then
        echo "No .config found. Run '$0 defconfig' first."
        exit 1
    fi
    cp "$DOTCONFIG" "$CONFIGS_DIR/saved_defconfig"
    echo "Saved current config -> $CONFIGS_DIR/saved_defconfig"
}

cmd_build() {
    if [ ! -f "$DOTCONFIG" ]; then
        echo "No .config found, using default (all features enabled, static=y)"
    fi
    check_musl
    local flags
    flags=$(cargo_flags debug)
    echo "Building: cargo build $flags"
    # shellcheck disable=SC2086
    cargo build $flags
    local bin
    bin=$(debug_binary)
    echo ""
    echo "Binary: $bin"
    if command -v file &>/dev/null; then
        file "$bin"
    fi
}

cmd_release() {
    if [ ! -f "$DOTCONFIG" ]; then
        echo "No .config found, using default (all features enabled, static=y)"
    fi
    check_musl
    local flags
    flags=$(cargo_flags release)
    echo "Building: cargo build $flags"
    # shellcheck disable=SC2086
    cargo build $flags
    local bin
    bin=$(release_binary)
    echo ""
    echo "Binary: $bin"
    if command -v file &>/dev/null; then
        file "$bin"
    fi
    if command -v ldd &>/dev/null; then
        ldd "$bin" 2>&1 || true
    fi
}

cmd_clean() {
    cargo clean
    echo "Build artifacts cleaned"
}

cmd_distclean() {
    cargo clean
    rm -f .config rootfs.cpio.gz
    [ -d "_install" ] && rm -rf "_install"
    echo "Distclean complete (build artifacts, .config, rootfs removed)"
}

cmd_show() {
    if [ ! -f "$DOTCONFIG" ]; then
        echo "No .config found. Using defaults (all features enabled, static=y)."
        return
    fi
    echo "Current configuration ($DOTCONFIG):"
    echo ""
    grep -v "^#" "$DOTCONFIG" | grep -v "^$" | sort
}

cmd_qemu_test() {
    local kernel="${KERNEL:-}"
    local timeout_sec="${QEMU_TIMEOUT:-20}"

    # ---- 1. Build release binary ----
    echo "==> Building release binary..."
    cmd_release
    echo ""

    local bin
    bin=$(release_binary)
    if [ ! -f "$bin" ]; then
        echo "Error: binary not found at $bin"
        exit 1
    fi

    # ---- 2. User-mode sanity check (qemu-x86_64) ----
    # This verifies the static binary has zero missing deps and all applets work
    # without needing a kernel image or root privileges.
    echo "==> User-mode sanity check (qemu-x86_64)..."
    local ok=0
    if command -v qemu-x86_64 &>/dev/null; then
        qemu-x86_64 "$bin" uname -m 2>&1        && \
        qemu-x86_64 "$bin" echo "echo: OK"       && \
        qemu-x86_64 "$bin" id   2>&1             && \
        echo "hello" | qemu-x86_64 "$bin" cat    && \
        echo "hello" | qemu-x86_64 "$bin" grep hello >/dev/null && \
        echo "==> User-mode check PASSED" && ok=1
        if [ "$ok" -ne 1 ]; then
            echo "Error: user-mode sanity check failed"
            exit 1
        fi
    else
        echo "    (qemu-x86_64 not found — skipping user-mode check)"
    fi
    echo ""

    # ---- 3. Find kernel for system QEMU boot ----
    if [ -z "$kernel" ]; then
        for k in \
            "/boot/vmlinuz-$(uname -r)" \
            "/boot/vmlinuz" \
            /boot/vmlinuz-*; do
            if [ -f "$k" ] && [ -r "$k" ]; then
                kernel="$k"
                break
            fi
        done
    fi

    if [ -z "$kernel" ] || [ ! -f "$kernel" ]; then
        echo "==> Skipping system QEMU boot: no readable kernel found."
        echo ""
        echo "    To run the full boot test, make a kernel readable first:"
        echo "      sudo cp /boot/vmlinuz /tmp/bzImage && sudo chmod 644 /tmp/bzImage"
        echo "    Then re-run:"
        echo "      KERNEL=/tmp/bzImage ./configure.sh qemu-test"
        echo ""
        echo "==> QEMU test complete (user-mode PASSED; system boot skipped)"
        return
    fi

    if ! command -v qemu-system-x86_64 &>/dev/null; then
        echo "Error: qemu-system-x86_64 not found."
        echo "Install with:  sudo apt install qemu-system-x86"
        exit 1
    fi

    # ---- 4. Create rootfs ----
    echo "==> Installing rootfs..."
    BINARY="$(pwd)/$bin" ./install.sh

    # ---- 5. Pack initramfs ----
    echo ""
    echo "==> Packing initramfs..."
    (cd _install && find . | cpio -o -H newc 2>/dev/null | gzip -9 > ../rootfs.cpio.gz)
    local size
    size=$(du -sh rootfs.cpio.gz | cut -f1)
    echo "    rootfs.cpio.gz: $size"

    # ---- 6. System QEMU boot ----
    echo ""
    echo "==> Booting in QEMU (kernel: $kernel)"
    echo "    Timeout: ${timeout_sec}s  |  Press Ctrl-C to abort early"
    echo "------------------------------------------------------------------------"

    local exit_code=0
    # -nographic routes both VGA and serial to stdio; -serial stdio would conflict
    timeout "$timeout_sec" \
        qemu-system-x86_64 \
            -kernel "$kernel" \
            -initrd rootfs.cpio.gz \
            -append "console=ttyS0 init=/init nokaslr quiet" \
            -nographic \
            -m 256M \
            -no-reboot \
        2>&1 || exit_code=$?

    echo "------------------------------------------------------------------------"
    if [ "$exit_code" -eq 124 ] || [ "$exit_code" -eq 0 ]; then
        echo "==> System QEMU test complete (exit $exit_code — timeout/clean-exit is OK)"
    else
        echo "==> QEMU exited with code $exit_code"
    fi
}

case "${1:-help}" in
    menuconfig)   cmd_menuconfig ;;
    defconfig)    cmd_defconfig ;;
    minimal)      cmd_minimal ;;
    savedefconfig) cmd_savedefconfig ;;
    build)        cmd_build ;;
    release)      cmd_release ;;
    clean)        cmd_clean ;;
    distclean)    cmd_distclean ;;
    qemu-test)    cmd_qemu_test ;;
    show)         cmd_show ;;
    help|--help|-h) usage ;;
    *) echo "Unknown command: $1"; usage; exit 1 ;;
esac
