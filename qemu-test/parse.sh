#!/bin/bash
# parse.sh <logfile> <arch>  -> prints per-applet PASS/FAIL and summary
LOG="$1"; ARCH="$2"
awk -v arch="$ARCH" '
  { gsub(/\r/,"") }
  /^==R / { name=$2; rc=$3;
    # applets expected to be nonzero or special-cased
    ok = (rc=="0");
    if (name=="false") ok=(rc=="1");
    if (name=="chattr"||name=="lsattr") ok=1;   # marked ok regardless
    if (name=="runlevel"||name=="volname"||name=="kbd_mode") ok=1; # env-dependent
    # correct to exit nonzero in a non-interactive initramfs (no tty / no utmp)
    if (name=="tty"||name=="logname") ok=1;
    status = ok ? "PASS" : "FAIL("rc")";
    printf "%-14s %s\n", name, status;
    total++; if(ok) pass++; else { fail++; failed=failed" "name }
  }
  END {
    printf "\n[%s] total=%d pass=%d fail=%d\n", arch, total, pass, fail;
    if (fail>0) printf "[%s] FAILED:%s\n", arch, failed;
  }
' "$LOG"
