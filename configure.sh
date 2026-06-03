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

usage() {
    echo "Usage: $0 <command>"
    echo ""
    echo "Configuration:"
    echo "  menuconfig     - Interactive configuration (requires python3 + kconfiglib)"
    echo "  defconfig      - Load default configuration (all features enabled)"
    echo "  minimal        - Load minimal configuration"
    echo "  savedefconfig  - Save current config as defconfig"
    echo ""
    echo "Build:"
    echo "  build          - Build debug binary"
    echo "  release        - Build release binary"
    echo "  clean          - Clean build artifacts"
    echo "  distclean      - Clean build artifacts, .config, and install directory"
    echo ""
    echo "Info:"
    echo "  show           - Show current configuration"
    echo "  help           - Show this help"
}

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
if kconf.load_config('.config') if __import__('os').path.exists('.config') else None:
    pass

kconf.load_config('.config' if __import__('os').path.exists('.config') else 'configs/default_defconfig')
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
        echo "No .config found, using default (all features enabled)"
    fi
    cargo build
}

cmd_release() {
    if [ ! -f "$DOTCONFIG" ]; then
        echo "No .config found, using default (all features enabled)"
    fi
    cargo build --release
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
        echo "No .config found. Using defaults (all features enabled)."
        return
    fi
    echo "Current configuration ($DOTCONFIG):"
    echo ""
    grep -v "^#" "$DOTCONFIG" | grep -v "^$" | sort
}

case "${1:-help}" in
    menuconfig) cmd_menuconfig ;;
    defconfig)  cmd_defconfig ;;
    minimal)    cmd_minimal ;;
    savedefconfig) cmd_savedefconfig ;;
    build)      cmd_build ;;
    release)    cmd_release ;;
    clean)      cmd_clean ;;
    distclean)  cmd_distclean ;;
    show)       cmd_show ;;
    help|--help|-h) usage ;;
    *) echo "Unknown command: $1"; usage; exit 1 ;;
esac
