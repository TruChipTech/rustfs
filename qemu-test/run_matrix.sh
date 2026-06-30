#!/bin/bash
# run_matrix.sh [arch...] -- build & qemu-system boot the full matrix:
#   arch in {x86_64, arm64, arm32, riscv64}
#   profile in {release, debug}
#   link in {static, shared}
# Results: work/results-<arch>-<profile>-<link>.txt ; summary -> work/MATRIX.txt
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$HERE")"
W="$HERE/work"
RV="$W/rvtc"; RVSR="$RV/sysroot/usr/riscv64-linux-gnu"
export PATH="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$RV/sysroot/usr/bin:$PATH"
export LD_LIBRARY_PATH="$RV/sysroot/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

ARCHES=("${@:-x86_64 arm64 arm32 riscv64}")
read -r -a ARCHES <<< "${ARCHES[*]}"
read -r -a PROFILES <<< "${RUN_PROFILES:-release debug}"
read -r -a LINKS <<< "${RUN_LINKS:-static shared}"

# cross C compilers / linkers for the various targets
export CC_x86_64_unknown_linux_musl=musl-gcc
export CC_aarch64_unknown_linux_musl=aarch64-linux-gnu-gcc
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc
export CC_armv7_unknown_linux_musleabihf=arm-linux-gnueabihf-gcc
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=arm-linux-gnueabihf-gcc
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc
# riscv (glibc cross toolchain extracted under rvtc)
export CC_riscv64gc_unknown_linux_gnu=riscv64-linux-gnu-gcc
export CFLAGS_riscv64gc_unknown_linux_gnu="--sysroot=$RVSR"
export AR_riscv64gc_unknown_linux_gnu=riscv64-linux-gnu-ar
export CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_LINKER=riscv64-linux-gnu-gcc

triple() { # arch link -> target triple
  case "$1/$2" in
    x86_64/static)  echo x86_64-unknown-linux-musl;;
    x86_64/shared)  echo x86_64-unknown-linux-gnu;;
    arm64/static)   echo aarch64-unknown-linux-musl;;
    arm64/shared)   echo aarch64-unknown-linux-gnu;;
    arm32/static)   echo armv7-unknown-linux-musleabihf;;
    arm32/shared)   echo armv7-unknown-linux-gnueabihf;;
    riscv64/static) echo riscv64gc-unknown-linux-gnu;;
    riscv64/shared) echo riscv64gc-unknown-linux-gnu;;
  esac
}
libdir() { # arch -> host dir holding that arch's glibc shared objects
  case "$1" in
    x86_64)  echo /lib/x86_64-linux-gnu;;
    arm64)   echo /usr/aarch64-linux-gnu/lib;;
    arm32)   echo /usr/arm-linux-gnueabihf/lib;;
    riscv64) echo "$RVSR/lib";;
  esac
}
qemu_base() {
  case "$1" in
    x86_64)  echo "qemu-system-x86_64 -M pc -cpu max -kernel $W/vmlinuz-amd64";;
    arm64)   echo "qemu-system-aarch64 -M virt -cpu cortex-a57 -kernel $W/Image-arm64";;
    arm32)   echo "qemu-system-arm -M virt -cpu cortex-a15 -kernel $W/vmlinuz-armhf";;
    riscv64) echo "qemu-system-riscv64 -M virt -kernel $W/Image-riscv64";;
  esac
}
console() { case "$1" in x86_64|riscv64) echo ttyS0;; arm64|arm32) echo ttyAMA0;; esac; }

# regenerate the applet symlink list once from a freshly built native binary
gen_applets() {
  local bin="$1"
  "$bin" --help 2>&1 | sed -n '/Available applets:/,$p' | tail -n +2 \
    | tr ',' ' ' | tr -s ' \t\n' '\n' | sed '/^$/d' | sort -u > "$W/applets.txt"
  printf 'mount\numount\n' >> "$W/applets.txt"; sort -u "$W/applets.txt" -o "$W/applets.txt"
}

SUMMARY="$W/MATRIX.txt"; : > "$SUMMARY"
for arch in "${ARCHES[@]}"; do
 for link in "${LINKS[@]}"; do
  for prof in "${PROFILES[@]}"; do
    tag="$arch-$prof-$link"
    tgt=$(triple "$arch" "$link")
    pflag=""; pdir="debug"; [ "$prof" = release ] && { pflag="--release"; pdir="release"; }

    # RUSTFLAGS for static riscv (crt-static + sysroot); shared riscv uses sysroot too
    RF=""
    if [ "$arch" = riscv64 ]; then
      if [ "$link" = static ]; then
        RF="-C target-feature=+crt-static -C link-arg=--sysroot=$RVSR -C link-arg=-static -L $RVSR/lib"
      else
        # For dynamic linking the libc.so linker script uses absolute paths, so
        # point --sysroot at the extraction root (not the per-triple subdir) to
        # avoid path doubling; -L adds libgcc_s/libcrypt from the triple libdir.
        RF="-C link-arg=--sysroot=$RV/sysroot -L $RVSR/lib"
      fi
    fi
    # glibc cross sysroots for arm lack libcrypt (needed by the login applet on
    # glibc targets); point the linker at the extracted libcrypt for shared builds.
    if [ "$link" = shared ]; then
      [ "$arch" = arm64 ] && RF="-L $W/xlibs/arm64/usr/lib/aarch64-linux-gnu"
      [ "$arch" = arm32 ] && RF="-L $W/xlibs/armhf/usr/lib/arm-linux-gnueabihf"
    fi

    echo "### BUILD $tag ($tgt)"
    if ! ( cd "$ROOT" && CARGO_BUILD_RUSTFLAGS="$RF" cargo build $pflag --target "$tgt" ) >"$W/build-$tag.log" 2>&1; then
      echo "$tag : BUILD FAILED (see build-$tag.log)" | tee -a "$SUMMARY"
      tail -3 "$W/build-$tag.log"
      continue
    fi
    bin="$ROOT/target/$tgt/$pdir/rustfs"

    # generate applet list from the matching-endianness binary once (use native release)
    [ -s "$W/applets.txt" ] || gen_applets "$ROOT/target/release/rustfs"

    img="$W/initramfs-$tag.cpio.gz"
    if [ "$link" = shared ]; then
      export RUSTFS_EXTRA_LIBDIR=""
      [ "$arch" = arm64 ] && export RUSTFS_EXTRA_LIBDIR="$W/xlibs/arm64/usr/lib/aarch64-linux-gnu"
      [ "$arch" = arm32 ] && export RUSTFS_EXTRA_LIBDIR="$W/xlibs/armhf/usr/lib/arm-linux-gnueabihf"
      "$HERE/mkinitramfs.sh" "$bin" "$img" "$(libdir "$arch")" >/dev/null
    else
      "$HERE/mkinitramfs.sh" "$bin" "$img" >/dev/null
    fi

    echo "### BOOT $tag"
    log="$W/log-$tag.txt"
    timeout 600 $(qemu_base "$arch") -append "console=$(console "$arch") panic=-1" \
      -m 512 -initrd "$img" -nographic -no-reboot -serial mon:stdio >"$log" 2>&1 || true

    if grep -aq '==END==' "$log"; then
      bash "$HERE/parse.sh" "$log" "$tag" > "$W/results-$tag.txt"
      grep "total=" "$W/results-$tag.txt" | tee -a "$SUMMARY"
    else
      echo "$tag : BOOT FAILED / no ==END==" | tee -a "$SUMMARY"
    fi
  done
 done
done
echo "=== MATRIX SUMMARY ==="; cat "$SUMMARY"
