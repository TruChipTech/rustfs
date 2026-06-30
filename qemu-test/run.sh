#!/bin/bash
# run.sh <arch>  -- build rustfs, make initramfs, boot in qemu-system, parse results
set -e
ARCH="$1"
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$HERE")"
W="$HERE/work"
export PATH="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"

case "$ARCH" in
  x86_64)  RTGT=x86_64-unknown-linux-musl;  QEMU="qemu-system-x86_64 -M pc -cpu max -kernel $W/vmlinuz-amd64";  APP="console=ttyS0";;
  arm64)   RTGT=aarch64-unknown-linux-musl; QEMU="qemu-system-aarch64 -M virt -cpu cortex-a57 -kernel $W/Image-arm64"; APP="console=ttyAMA0";;
  arm32)   RTGT=armv7-unknown-linux-musleabihf; QEMU="qemu-system-arm -M virt -cpu cortex-a15 -kernel $W/vmlinuz-armhf"; APP="console=ttyAMA0";;
  riscv64) RTGT=riscv64gc-unknown-linux-gnu; QEMU="qemu-system-riscv64 -M virt -kernel $W/Image-riscv64"; APP="console=ttyS0";;
  *) echo "unknown arch $ARCH"; exit 1;;
esac

echo "### [$ARCH] building ($RTGT)"
( cd "$ROOT"
  export CC_x86_64_unknown_linux_musl=musl-gcc
  export CC_aarch64_unknown_linux_musl=aarch64-linux-gnu-gcc
  export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc
  export CC_armv7_unknown_linux_musleabihf=arm-linux-gnueabihf-gcc
  export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=arm-linux-gnueabihf-gcc
  cargo build --release --target "$RTGT" 2>&1 | tail -2 )

BIN="$ROOT/target/$RTGT/release/rustfs"
file "$BIN"
"$HERE/mkinitramfs.sh" "$BIN" "$W/initramfs-$ARCH.cpio.gz"

echo "### [$ARCH] booting qemu-system"
timeout 600 $QEMU -m 512 -initrd "$W/initramfs-$ARCH.cpio.gz" \
  -append "$APP panic=-1" -nographic -no-reboot -serial mon:stdio > "$W/log-$ARCH.txt" 2>&1 || true

echo "### [$ARCH] results"
if grep -aq '==END==' "$W/log-$ARCH.txt"; then
  bash "$HERE/parse.sh" "$W/log-$ARCH.txt" "$ARCH" > "$W/results-$ARCH.txt"
  tail -3 "$W/results-$ARCH.txt"
else
  echo "[$ARCH] BOOT FAILED or test did not complete (no ==END==). Tail:"
  tail -15 "$W/log-$ARCH.txt"
fi
