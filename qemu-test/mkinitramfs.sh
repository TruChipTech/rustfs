#!/bin/bash
# Usage: mkinitramfs.sh <rustfs-binary> <output-initramfs.cpio.gz> [shared-libdir]
#
# If <shared-libdir> is given, the binary is dynamically linked: the ELF
# interpreter (dynamic loader) and every .so* in <shared-libdir> are bundled
# into the image so the binary can run inside the minimal rootfs.
set -e
BIN="$1"; OUT="$2"; LIBDIR="$3"
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(mktemp -d)"
trap 'rm -rf "$ROOT"' EXIT

mkdir -p "$ROOT"/{bin,proc,sys,dev,tmp,t,etc,lib,lib64,usr/lib}
printf 'root:x:0:0:root:/root:/bin/sh\n' > "$ROOT/etc/passwd"
printf 'root:x:0:\n' > "$ROOT/etc/group"
printf 'rustfs-qemu\n' > "$ROOT/etc/hostname"
cp "$BIN" "$ROOT/bin/rustfs"
chmod +x "$ROOT/bin/rustfs"
# multi-call: symlink every applet name to the multicall binary
while read -r app; do
  [ -n "$app" ] && ln -sf rustfs "$ROOT/bin/$app"
done < "$HERE/work/applets.txt"
ln -sf rustfs "$ROOT/bin/sh"
cp "$HERE/applet_tests.sh" "$ROOT/applet_tests.sh"

LDPATH=""
if [ -n "$LIBDIR" ]; then
  SEARCH=("$LIBDIR" "$(dirname "$LIBDIR")" /lib64 /lib /usr/lib)
  # Optional extra search dir (e.g. for libs not present in the cross sysroot).
  [ -n "${RUSTFS_EXTRA_LIBDIR:-}" ] && SEARCH=("${RUSTFS_EXTRA_LIBDIR}" "${SEARCH[@]}")
  find_lib() { # locate a soname across the search dirs
    local name="$1" d
    for d in "${SEARCH[@]}"; do
      local hit
      hit=$(find "$d" -maxdepth 2 -name "$name" 2>/dev/null | head -1)
      [ -n "$hit" ] && { echo "$hit"; return; }
    done
  }
  needed_of() { readelf -d "$1" 2>/dev/null | sed -n 's/.*(NEEDED).*\[\(.*\)\]/\1/p'; }

  # Place the ELF interpreter (dynamic loader) at the path the binary expects.
  INTERP=$(readelf -l "$BIN" 2>/dev/null | sed -n 's/.*interpreter: \(.*\)\]/\1/p' | tr -d ' ')
  if [ -n "$INTERP" ]; then
    mkdir -p "$ROOT$(dirname "$INTERP")"
    LDSRC=$(find_lib "$(basename "$INTERP")")
    [ -n "$LDSRC" ] && cp -L "$LDSRC" "$ROOT$INTERP"
  fi

  # Copy only the binary's dependency closure (transitive NEEDED libraries).
  declare -A seen=()
  queue=$(needed_of "$BIN")
  while [ -n "$queue" ]; do
    next=""
    for so in $queue; do
      [ -n "${seen[$so]:-}" ] && continue
      seen[$so]=1
      src=$(find_lib "$so")
      if [ -n "$src" ]; then
        cp -Ln "$src" "$ROOT/lib/" 2>/dev/null || true
        next="$next $(needed_of "$src")"
      fi
    done
    queue="$next"
  done
  LDPATH="export LD_LIBRARY_PATH=/lib:/lib64"
fi

cat > "$ROOT/init" <<EOF
#!/bin/sh
export PATH=/bin
$LDPATH
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
