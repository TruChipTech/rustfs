#!/bin/bash
# add_applet.sh <applet-name> ["alias1 alias2 ..."]
# Wires an already-written src/applets/<module>.rs into all integration points:
#   - src/applets/mod.rs        (#[cfg(applet_X)] pub mod X;)
#   - src/main.rs               (dispatch arm + help-list entry)
#   - Cargo.toml                (check-cfg 'cfg(applet_X)')
#   - Kconfig                   (config APPLET_X block)
#   - configs/default_defconfig (CONFIG_APPLET_X=y)
# Names with '.'/'-' map to '_' for the Rust module / cfg / CONFIG symbol, while
# the dispatch string keeps the original applet name.
set -e
cd "$(dirname "$0")/.."

NAME="$1"        # real applet name, e.g. run-parts, fsck.minix
ALIASES="$2"     # optional space-separated extra dispatch names
MOD="${NAME//[.-]/_}"          # module / cfg base: run_parts
CFG="applet_${MOD}"
SYM="APPLET_${MOD^^}"
RSFILE="src/applets/${MOD}.rs"

[ -f "$RSFILE" ] || { echo "ERROR: $RSFILE not found (write the impl first)"; exit 1; }

# 1. mod.rs
if ! grep -q "pub mod ${MOD};" src/applets/mod.rs; then
  printf '#[cfg(%s)]\npub mod %s;\n' "$CFG" "$MOD" >> src/applets/mod.rs
fi

# 2. Cargo.toml check-cfg (insert before closing "] }")
if ! grep -q "cfg(${CFG})" Cargo.toml; then
  awk -v c="    'cfg(${CFG})'," '
    /^\] }$/ && !done { print c; done=1 }
    { print }
  ' Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml
fi

# 3. main.rs dispatch arm (insert before the "// Help / meta" marker)
if ! grep -q "applets::${MOD}::run" src/main.rs; then
  pat="\"${NAME}\""
  for a in $ALIASES; do pat="$pat | \"${a}\""; done
  arm="        #[cfg(${CFG})]\n        ${pat} => applets::${MOD}::run(\&applet_args),"
  awk -v arm="$arm" '
    /\/\/ Help \/ meta/ && !done { gsub(/\\n/,"\n",arm); print arm; done=1 }
    { print }
  ' src/main.rs > src/main.rs.tmp && mv src/main.rs.tmp src/main.rs
fi

# 4. main.rs help list (insert "<name>", before the closing "];" of the array)
if ! grep -q "\"${NAME}\"," src/main.rs; then
  entry="        \"${NAME}\","
  awk -v e="$entry" '
    /^    \];$/ && !done { print e; done=1 }
    { print }
  ' src/main.rs > src/main.rs.tmp && mv src/main.rs.tmp src/main.rs
fi

# 5. Kconfig (append a config block at end if absent)
if ! grep -q "^config ${SYM}\$" Kconfig; then
  {
    echo ""
    echo "config ${SYM}"
    echo "	bool \"${NAME}\""
    echo "	default y"
    echo "	help"
    echo "	  Enable the ${NAME} applet."
  } >> Kconfig
fi

# 6. default_defconfig
if ! grep -q "^CONFIG_${SYM}=y" configs/default_defconfig; then
  echo "CONFIG_${SYM}=y" >> configs/default_defconfig
fi

# 7. build.rs all_options[] (so build.rs emits the cfg flag from .config)
if ! grep -q "\"${SYM}\"," build.rs; then
  awk -v s="        \"${SYM}\"," '
    /^    let all_options = \[$/ && !done { print; print s; done=1; next }
    { print }
  ' build.rs > build.rs.tmp && mv build.rs.tmp build.rs
fi

echo "wired: ${NAME}  (mod=${MOD} cfg=${CFG} sym=${SYM})"
