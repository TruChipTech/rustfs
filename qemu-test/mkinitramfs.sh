#!/bin/bash
# Usage: mkinitramfs.sh <rustfs-binary> <output-initramfs.cpio.gz>
set -e
BIN="$1"; OUT="$2"
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(mktemp -d)"
trap 'rm -rf "$ROOT"' EXIT

mkdir -p "$ROOT"/{bin,proc,sys,dev,tmp,t,etc}
printf 'root:x:0:0:root:/root:/bin/sh\n' > "$ROOT/etc/passwd"
printf 'root:x:0:\n' > "$ROOT/etc/group"
printf 'rustfs-qemu\n' > "$ROOT/etc/hostname"
cp "$BIN" "$ROOT/bin/rustfs"
chmod +x "$ROOT/bin/rustfs"
# busybox-style: symlink every applet name to the multicall binary
while read -r app; do
  [ -n "$app" ] && ln -sf rustfs "$ROOT/bin/$app"
done < "$HERE/work/applets.txt"
ln -sf rustfs "$ROOT/bin/sh"
cp "$HERE/applet_tests.sh" "$ROOT/applet_tests.sh"

cat > "$ROOT/init" <<'EOF'
#!/bin/sh
export PATH=/bin
/bin/rustfs mount -t proc proc /proc
/bin/rustfs mount -t sysfs sysfs /sys
/bin/rustfs mount -t devtmpfs devtmpfs /dev 2>/dev/null
/bin/rustfs mount -t tmpfs tmpfs /tmp 2>/dev/null
/bin/rustfs echo "##RUSTFS-TEST-START##"
/bin/rustfs sh /applet_tests.sh
/bin/rustfs echo "##RUSTFS-TEST-DONE##"
/bin/rustfs sync
/bin/rustfs echo o > /proc/sysrq-trigger
/bin/rustfs sleep 5
EOF
chmod +x "$ROOT/init"

( cd "$ROOT" && find . | cpio -o -H newc 2>/dev/null | gzip -9 ) > "$OUT"
echo "initramfs -> $OUT ($(du -h "$OUT" | cut -f1))"
