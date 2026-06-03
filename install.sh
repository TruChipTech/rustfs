#!/bin/sh
# <purpose of file>
#
# Copyright (C) 2026 by Anandkumar  <Truchip >
#
# Licensed under MIT
# install.sh — Install RustFS into a minimal rootfs
#
# Usage: ./install.sh [OPTIONS]
#
# Options:
#   INSTALL_DIR=<path>   Root installation directory (default: _install)
#   clean                Remove the install directory
#   distclean            Remove install directory, .config, and generated artifacts
#
# Creates a complete minimal rootfs with directory structure, device nodes,
# init scripts, and device manager configuration based on .config.
#
# Examples:
#   ./install.sh
#   ./install.sh INSTALL_DIR=/tmp/myrootfs
#   ./install.sh clean
#   ./install.sh distclean

set -e

# Parse arguments
ACTION="install"
for arg in "$@"; do
    case "$arg" in
        INSTALL_DIR=*) INSTALL_DIR="${arg#INSTALL_DIR=}" ;;
        clean) ACTION="clean" ;;
        distclean) ACTION="distclean" ;;
        -h|--help)
            sed -n '2,/^$/p' "$0" | grep '^#' | sed 's/^# \?//'
            exit 0 ;;
        *) echo "Unknown option: $arg"; exit 1 ;;
    esac
done

INSTALL_DIR="${INSTALL_DIR:-_install}"

remove_install_dir() {
    if [ ! -d "$INSTALL_DIR" ]; then
        return 0
    fi

    if [ "$(id -u)" = "0" ]; then
        rm -rf "$INSTALL_DIR"
    elif command -v sudo >/dev/null 2>&1; then
        sudo rm -rf "$INSTALL_DIR"
    else
        echo "Error: cannot remove $INSTALL_DIR without root privileges or sudo"
        return 1
    fi
}

# Handle clean action
if [ "$ACTION" = "clean" ]; then
    if [ -d "$INSTALL_DIR" ]; then
        if remove_install_dir; then
            echo "Removed $INSTALL_DIR"
        else
            exit 1
        fi
    else
        echo "Nothing to clean ($INSTALL_DIR does not exist)"
    fi
    exit 0
fi

# Handle distclean action
if [ "$ACTION" = "distclean" ]; then
    if [ -d "$INSTALL_DIR" ]; then
        if remove_install_dir; then
            echo "Removed $INSTALL_DIR"
        else
            exit 1
        fi
    fi
    [ -f ".config" ] && rm -f ".config" && echo "Removed .config"
    [ -f "rootfs.cpio.gz" ] && rm -f "rootfs.cpio.gz" && echo "Removed rootfs.cpio.gz"
    echo "Distclean complete"
    exit 0
fi

# Read configuration
CONFIG_FILE=""
if [ -f ".config" ]; then
    CONFIG_FILE=".config"
elif [ -f "configs/default_defconfig" ]; then
    CONFIG_FILE="configs/default_defconfig"
fi

config_enabled() {
    [ -n "$CONFIG_FILE" ] && grep -q "^CONFIG_$1=y" "$CONFIG_FILE"
}

# Check binary
BINARY="${BINARY:-$(cd "$(dirname "$0")" && pwd)/target/release/rustfs}"
if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    echo "Run 'cargo build --release' first."
    exit 1
fi

echo "Installing RustFS rootfs at: $INSTALL_DIR"
echo "  Config: ${CONFIG_FILE:-<none, defaults>}"
echo

# ============================================================
# 1. Create directory structure
# ============================================================
echo "Creating directory structure..."
mkdir -p "$INSTALL_DIR"/{bin,sbin,usr/bin,usr/sbin,usr/lib}
mkdir -p "$INSTALL_DIR"/{etc,etc/init.d,etc/network}
mkdir -p "$INSTALL_DIR"/{dev,proc,sys,tmp,run,var/log,var/run,var/tmp}
mkdir -p "$INSTALL_DIR"/{root,home,mnt,opt,lib}
chmod 1777 "$INSTALL_DIR/tmp"
chmod 700 "$INSTALL_DIR/root"

# ============================================================
# 2. Create essential device nodes
# ============================================================
echo "Creating device nodes..."
if [ "$(id -u)" = "0" ]; then
    mknod -m 666 "$INSTALL_DIR/dev/null" c 1 3 2>/dev/null || true
    mknod -m 666 "$INSTALL_DIR/dev/zero" c 1 5 2>/dev/null || true
    mknod -m 666 "$INSTALL_DIR/dev/full" c 1 7 2>/dev/null || true
    mknod -m 666 "$INSTALL_DIR/dev/random" c 1 8 2>/dev/null || true
    mknod -m 666 "$INSTALL_DIR/dev/urandom" c 1 9 2>/dev/null || true
    mknod -m 666 "$INSTALL_DIR/dev/tty" c 5 0 2>/dev/null || true
    mknod -m 620 "$INSTALL_DIR/dev/console" c 5 1 2>/dev/null || true
    mknod -m 660 "$INSTALL_DIR/dev/ttyS0" c 4 64 2>/dev/null || true
    mknod -m 660 "$INSTALL_DIR/dev/tty0" c 4 0 2>/dev/null || true
    mknod -m 660 "$INSTALL_DIR/dev/tty1" c 4 1 2>/dev/null || true
    mkdir -p "$INSTALL_DIR/dev/pts" "$INSTALL_DIR/dev/shm"
else
    echo "  (skipping device nodes — not running as root)"
    mkdir -p "$INSTALL_DIR/dev/pts" "$INSTALL_DIR/dev/shm"
fi

# ============================================================
# 3. Install RustFS binary and create symlinks
# ============================================================
echo "Installing binary and symlinks..."
cp "$BINARY" "$INSTALL_DIR/bin/rustfs"
chmod +x "$INSTALL_DIR/bin/rustfs"

# init symlink at root level
ln -sf bin/rustfs "$INSTALL_DIR/init"

# Applets for bin/
BIN_APPLETS="ar awk base64 basename bunzip2 bzcat bzip2 cat chgrp chown chmod
clear cp cut date dd df diff dirname dos2unix du echo env expr false find
fold getopt grep gzip gunzip hd head hexdump hostid hostname id install
last length less ln logname ls md5sum mkdir mv nl nohup paste printenv
printf pwd readlink realpath rev rm rmdir sed seq sh sha256sum sleep sort
stat tail tee test touch tr true tty uname uniq uptime wc which whoami
xargs xxd yes"

# Special symlink: [ -> rustfs (test applet)
ln -sf rustfs "$INSTALL_DIR/bin/[" 2>/dev/null || true
# ash alias
ln -sf rustfs "$INSTALL_DIR/bin/ash" 2>/dev/null || true

# Applets for sbin/
SBIN_APPLETS="addgroup adduser arp arping blkid delgroup deluser dmesg
fbset fdisk fsck fsync ftpd ftpget ftpput fuser getty halt httpd hwclock
ifconfig ifdown ifup init insmod ip ipaddr ipcalc ipcrm ipcs kill killall
klogd logger login logread losetup lsmod mdev modinfo modprobe mount
poweroff reboot rmmod depmod umount"

bin_count=0
for applet in $BIN_APPLETS; do
    ln -sf rustfs "$INSTALL_DIR/bin/$applet"
    bin_count=$((bin_count + 1))
done

sbin_count=0
for applet in $SBIN_APPLETS; do
    ln -sf ../bin/rustfs "$INSTALL_DIR/sbin/$applet"
    sbin_count=$((sbin_count + 1))
done

echo "  $bin_count applets -> bin/"
echo "  $sbin_count applets -> sbin/"

# ============================================================
# 3b. Install dynamic libraries (if binary is dynamically linked)
# ============================================================
if command -v ldd >/dev/null 2>&1 && ldd "$BINARY" >/dev/null 2>&1; then
    if ldd "$BINARY" | grep -q "not a dynamic executable"; then
        echo "Binary is statically linked, skipping library install."
    else
        echo "Installing dynamic libraries..."
        lib_count=0
        ldd "$BINARY" | grep -o '/[^ ]*' | while read -r lib; do
            if [ -f "$lib" ]; then
                lib_dir=$(dirname "$lib")
                mkdir -p "$INSTALL_DIR$lib_dir"
                cp "$lib" "$INSTALL_DIR$lib" 2>/dev/null && lib_count=$((lib_count + 1))
            fi
        done
        # Copy the dynamic linker
        interp=$(readelf -l "$BINARY" 2>/dev/null | grep "interpreter" | sed 's/.*: \(.*\)]/\1/')
        if [ -n "$interp" ] && [ -f "$interp" ]; then
            interp_dir=$(dirname "$interp")
            mkdir -p "$INSTALL_DIR$interp_dir"
            cp "$interp" "$INSTALL_DIR$interp" 2>/dev/null
        fi
        echo "  dynamic libraries installed"
    fi
fi

# ============================================================
# 4. Create base configuration files
# ============================================================
echo "Creating configuration files..."

cat > "$INSTALL_DIR/etc/passwd" << 'EOF'
root:x:0:0:root:/root:/bin/sh
daemon:x:1:1:daemon:/usr/sbin:/bin/false
nobody:x:65534:65534:nobody:/nonexistent:/bin/false
EOF

cat > "$INSTALL_DIR/etc/group" << 'EOF'
root:x:0:
daemon:x:1:
tty:x:5:
disk:x:6:
kmem:x:9:
wheel:x:10:root
nogroup:x:65534:
EOF

cat > "$INSTALL_DIR/etc/shadow" << 'EOF'
root::0:0:99999:7:::
daemon:*:0:0:99999:7:::
nobody:*:0:0:99999:7:::
EOF
chmod 640 "$INSTALL_DIR/etc/shadow"

echo "rustfs" > "$INSTALL_DIR/etc/hostname"

cat > "$INSTALL_DIR/etc/hosts" << 'EOF'
127.0.0.1	localhost
127.0.1.1	rustfs
::1		localhost ip6-localhost ip6-loopback
EOF

cat > "$INSTALL_DIR/etc/fstab" << 'EOF'
# <filesystem>  <mount point>  <type>  <options>          <dump> <pass>
proc            /proc          proc    defaults           0      0
sysfs           /sys           sysfs   defaults           0      0
devtmpfs        /dev           devtmpfs defaults          0      0
devpts          /dev/pts       devpts  defaults,gid=5,mode=620 0 0
tmpfs           /tmp           tmpfs   defaults,nosuid    0      0
tmpfs           /run           tmpfs   defaults           0      0
EOF

cat > "$INSTALL_DIR/etc/profile" << 'EOF'
export PATH="/bin:/sbin:/usr/bin:/usr/sbin"
export HOME="${HOME:-/root}"
export PS1='[\u@\h \W]\$ '
export TERM="${TERM:-linux}"
EOF

cat > "$INSTALL_DIR/etc/shells" << 'EOF'
/bin/sh
EOF

cat > "$INSTALL_DIR/etc/network/interfaces" << 'EOF'
# Loopback
auto lo
iface lo inet loopback

# Primary network interface
auto eth0
iface eth0 inet dhcp
EOF

cat > "$INSTALL_DIR/etc/resolv.conf" << 'EOF'
nameserver 8.8.8.8
nameserver 8.8.4.4
EOF

# ============================================================
# 5. Configure init system based on .config
# ============================================================

# --- RustFS Init ---
if config_enabled "INIT_RUSTFS" || [ -z "$CONFIG_FILE" ]; then
    echo "Configuring RustFS init..."

    cat > "$INSTALL_DIR/etc/inittab" << 'EOF'
# RustFS init configuration
# Format: id::action:command
#
# Note: Essential filesystems (proc, sys, dev, etc.) are mounted
# automatically by init before processing this file.

# Set hostname
null::sysinit:/bin/hostname -F /etc/hostname

# Create runtime directories
null::sysinit:/bin/mkdir -p /var/log /var/run /var/tmp

# Start device manager
null::sysinit:/sbin/mdev -s

# Bring up loopback
null::sysinit:/sbin/ifconfig lo 127.0.0.1 up

# Start getty on console
ttyS0::respawn:/sbin/getty 115200 ttyS0
tty1::respawn:/sbin/getty 38400 tty1

# Shutdown/restart
null::shutdown:/bin/echo "Shutting down..."
null::restart:/sbin/init
null::ctrlaltdel:/sbin/reboot
EOF

    cat > "$INSTALL_DIR/etc/init.d/rcS" << 'EOF'
#!/bin/sh
# RustFS system initialization script

echo "RustFS booting..."

export PATH="/bin:/sbin:/usr/bin:/usr/sbin"

# Mount filesystems from fstab (if not already mounted)
mount -a 2>/dev/null

# Create necessary runtime dirs
mkdir -p /var/log /var/run /var/tmp

# Set hostname
if [ -f /etc/hostname ]; then
    hostname -F /etc/hostname
fi

# Start device manager if available
if [ -x /sbin/mdev ]; then
    echo "Starting mdev..."
    mdev -s
    echo /sbin/mdev > /proc/sys/kernel/hotplug
fi

# Load kernel modules if needed
if [ -d /etc/modules.d ]; then
    for mod in /etc/modules.d/*; do
        [ -f "$mod" ] && while read -r m; do
            [ -n "$m" ] && insmod "$m" 2>/dev/null
        done < "$mod"
    done
fi

# Bring up loopback interface
if [ -x /sbin/ifconfig ]; then
    ifconfig lo 127.0.0.1 up
fi

# Run local startup scripts
if [ -d /etc/init.d ]; then
    for script in /etc/init.d/S??*; do
        [ -x "$script" ] && "$script" start
    done
fi

echo "System ready."
EOF
    chmod +x "$INSTALL_DIR/etc/init.d/rcS"
fi

# --- System V Init ---
if config_enabled "INIT_SYSVINIT"; then
    echo "Configuring System V init..."

    cat > "$INSTALL_DIR/etc/inittab" << 'EOF'
# System V init configuration
# Format: id:runlevels:action:command

# Default runlevel
id:3:initdefault:

# System initialization
si::sysinit:/etc/init.d/rcS

# Runlevel scripts
l0:0:wait:/etc/init.d/rc 0
l1:1:wait:/etc/init.d/rc 1
l2:2:wait:/etc/init.d/rc 2
l3:3:wait:/etc/init.d/rc 3
l4:4:wait:/etc/init.d/rc 4
l5:5:wait:/etc/init.d/rc 5
l6:6:wait:/etc/init.d/rc 6

# Spawn getty
1:2345:respawn:/sbin/getty 38400 tty1
2:2345:respawn:/sbin/getty 38400 tty2
S0:2345:respawn:/sbin/getty 115200 ttyS0

# Ctrl-Alt-Del
ca::ctrlaltdel:/sbin/reboot

# Shutdown
pf::powerwait:/sbin/halt
EOF

    cat > "$INSTALL_DIR/etc/init.d/rcS" << 'EOF'
#!/bin/sh
# System V initialization
#
# Note: Essential filesystems (proc, sys, dev, devpts, run, shm)
# are already mounted by init before this script runs.

export PATH="/bin:/sbin:/usr/bin:/usr/sbin"

echo "Setting hostname..."
hostname -F /etc/hostname

echo "Creating runtime directories..."
mkdir -p /var/log /var/run /var/tmp /tmp

echo "Starting device manager..."
mdev -s

echo "Bringing up loopback..."
ifconfig lo 127.0.0.1 up

echo "System initialization complete."
EOF
    chmod +x "$INSTALL_DIR/etc/init.d/rcS"

    # Runlevel control script
    cat > "$INSTALL_DIR/etc/init.d/rc" << 'EOF'
#!/bin/sh
# Runlevel change script
RUNLEVEL="$1"
echo "Entering runlevel $RUNLEVEL"

# Kill scripts
for script in /etc/rc${RUNLEVEL}.d/K??*; do
    test -x "$script" && "$script" stop
done
# Start scripts
for script in /etc/rc${RUNLEVEL}.d/S??*; do
    test -x "$script" && "$script" start
done
EOF
    chmod +x "$INSTALL_DIR/etc/init.d/rc"

    # Create runlevel directories
    for i in 0 1 2 3 4 5 6; do
        mkdir -p "$INSTALL_DIR/etc/rc${i}.d"
    done
fi

# --- systemd Init ---
if config_enabled "INIT_SYSTEMD"; then
    echo "Configuring systemd-compatible init..."

    mkdir -p "$INSTALL_DIR/etc/systemd/system"
    mkdir -p "$INSTALL_DIR/usr/lib/systemd/system"
    mkdir -p "$INSTALL_DIR/etc/systemd/system/multi-user.target.wants"

    cat > "$INSTALL_DIR/etc/systemd/system/default.target" << 'EOF'
[Unit]
Description=Default Target
Requires=multi-user.target
After=multi-user.target
EOF

    cat > "$INSTALL_DIR/usr/lib/systemd/system/multi-user.target" << 'EOF'
[Unit]
Description=Multi-User System
Requires=basic.target
After=basic.target
EOF

    cat > "$INSTALL_DIR/usr/lib/systemd/system/basic.target" << 'EOF'
[Unit]
Description=Basic System
Requires=sysinit.target
After=sysinit.target
EOF

    cat > "$INSTALL_DIR/usr/lib/systemd/system/sysinit.target" << 'EOF'
[Unit]
Description=System Initialization
EOF

    cat > "$INSTALL_DIR/usr/lib/systemd/system/setup-runtime.service" << 'EOF'
[Unit]
Description=Create Runtime Directories

[Service]
Type=oneshot
ExecStart=/bin/mkdir -p /var/log /var/run /var/tmp /tmp

[Install]
WantedBy=multi-user.target
EOF

    cat > "$INSTALL_DIR/usr/lib/systemd/system/hostname.service" << 'EOF'
[Unit]
Description=Set Hostname

[Service]
Type=oneshot
ExecStart=/bin/hostname -F /etc/hostname

[Install]
WantedBy=multi-user.target
EOF

    cat > "$INSTALL_DIR/usr/lib/systemd/system/mdev.service" << 'EOF'
[Unit]
Description=Device Manager (mdev)
After=setup-runtime.service

[Service]
Type=oneshot
ExecStart=/sbin/mdev -s

[Install]
WantedBy=multi-user.target
EOF

    cat > "$INSTALL_DIR/usr/lib/systemd/system/getty-ttyS0.service" << 'EOF'
[Unit]
Description=Getty on ttyS0
After=hostname.service mdev.service network-loopback.service

[Service]
Type=simple
ExecStart=/sbin/getty 115200 ttyS0
Restart=always

[Install]
WantedBy=multi-user.target
EOF

    cat > "$INSTALL_DIR/usr/lib/systemd/system/network-loopback.service" << 'EOF'
[Unit]
Description=Configure Loopback Interface

[Service]
Type=oneshot
ExecStart=/sbin/ifconfig lo 127.0.0.1 up

[Install]
WantedBy=multi-user.target
EOF

    ln -sf /usr/lib/systemd/system/setup-runtime.service \
        "$INSTALL_DIR/etc/systemd/system/multi-user.target.wants/"
    ln -sf /usr/lib/systemd/system/hostname.service \
        "$INSTALL_DIR/etc/systemd/system/multi-user.target.wants/"
    ln -sf /usr/lib/systemd/system/mdev.service \
        "$INSTALL_DIR/etc/systemd/system/multi-user.target.wants/"
    ln -sf /usr/lib/systemd/system/getty-ttyS0.service \
        "$INSTALL_DIR/etc/systemd/system/multi-user.target.wants/"
    ln -sf /usr/lib/systemd/system/network-loopback.service \
        "$INSTALL_DIR/etc/systemd/system/multi-user.target.wants/"
fi

# ============================================================
# 6. Configure device manager
# ============================================================

# --- mdev ---
if config_enabled "MDEV" || [ -z "$CONFIG_FILE" ]; then
    echo "Configuring mdev..."

    cat > "$INSTALL_DIR/etc/mdev.conf" << 'EOF'
# mdev configuration file
# Syntax: <device regex> <uid>:<gid> <permissions> [=path] [@|$|*cmd]

# Null/zero/full
null        0:0 0666
zero        0:0 0666
full        0:0 0666

# Random
random      0:0 0666
urandom     0:0 0666

# TTY devices
console     0:5 0600
tty         0:5 0666
tty[0-9]*   0:5 0660
ttyS[0-9]*  0:5 0660
ptmx        0:5 0666

# Input
input/.*    0:0 0660

# Block devices
sd[a-z].*   0:6 0660
vd[a-z].*   0:6 0660
mmcblk.*    0:6 0660
loop[0-9]*  0:6 0660

# Network
net/.*      0:0 0600

# USB
bus/usb/.*  0:0 0660
EOF
fi

# --- udev ---
if config_enabled "UDEV"; then
    echo "Configuring udev..."
    mkdir -p "$INSTALL_DIR/etc/udev/rules.d"
    mkdir -p "$INSTALL_DIR/usr/lib/udev/rules.d"
    mkdir -p "$INSTALL_DIR/run/udev"

    cat > "$INSTALL_DIR/etc/udev/rules.d/50-default.rules" << 'EOF'
# Console and TTY
KERNEL=="console", MODE="0600"
KERNEL=="tty", MODE="0666"
KERNEL=="tty[0-9]*", MODE="0660", GROUP="tty"
KERNEL=="ttyS[0-9]*", MODE="0660", GROUP="tty"

# Null, zero, full, random
KERNEL=="null", MODE="0666"
KERNEL=="zero", MODE="0666"
KERNEL=="full", MODE="0666"
KERNEL=="random", MODE="0666"
KERNEL=="urandom", MODE="0666"

# Block devices
KERNEL=="sd[a-z]*", GROUP="disk", MODE="0660"
KERNEL=="vd[a-z]*", GROUP="disk", MODE="0660"
KERNEL=="loop[0-9]*", GROUP="disk", MODE="0660"

# Network
SUBSYSTEM=="net", ACTION=="add", KERNEL=="eth*", NAME="eth%n"
SUBSYSTEM=="net", ACTION=="add", KERNEL=="wlan*", NAME="wlan%n"
EOF
fi

# ============================================================
# 7. Create shutdown/reboot scripts
# ============================================================
# Remove symlinks first to avoid overwriting the actual binary
rm -f "$INSTALL_DIR/sbin/halt" "$INSTALL_DIR/sbin/poweroff"
cat > "$INSTALL_DIR/sbin/halt" << 'EOF'
#!/bin/sh
echo "System halting..."
kill -TERM -1
sleep 1
kill -KILL -1
umount -a -r 2>/dev/null
sync
exec reboot -f
EOF
chmod +x "$INSTALL_DIR/sbin/halt"

cat > "$INSTALL_DIR/sbin/poweroff" << 'EOF'
#!/bin/sh
echo "Powering off..."
kill -TERM -1
sleep 1
kill -KILL -1
umount -a -r 2>/dev/null
sync
echo o > /proc/sysrq-trigger
EOF
chmod +x "$INSTALL_DIR/sbin/poweroff"

# ============================================================
# Summary
# ============================================================
echo
echo "============================================"
echo "RustFS rootfs installed at: $INSTALL_DIR"
echo "============================================"
echo
echo "  bin/:  $bin_count applets"
echo "  sbin/: $sbin_count applets"
echo "  init:  $INSTALL_DIR/init -> bin/rustfs"
echo
echo "To create a bootable initramfs:"
echo "  cd $INSTALL_DIR && find . | cpio -o -H newc | gzip > ../rootfs.cpio.gz"
echo
echo "To test with QEMU (x86_64):"
echo "  qemu-system-x86_64 -kernel /path/to/bzImage -initrd rootfs.cpio.gz \\"
echo "    -append 'console=ttyS0 init=/init' -nographic -m 256M"
echo
echo "To clean:"
echo "  ./install.sh clean"
