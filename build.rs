/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn main() {
    let config_path = Path::new(".config");

    // If no .config exists, enable everything by default
    let configs = if config_path.exists() {
        parse_dotconfig(config_path)
    } else {
        // Try configs/default_defconfig as fallback
        let defconfig = Path::new("configs/default_defconfig");
        if defconfig.exists() {
            parse_dotconfig(defconfig)
        } else {
            HashMap::new()
        }
    };

    // Emit cfg flags for each enabled config option
    let all_options = [
        "APPLET_RDATE",
        "APPLET_KBD_MODE",
        "APPLET_DEALLOCVT",
        "APPLET_CHVT",
        "APPLET_SETARCH",
        "APPLET_BEEP",
        "APPLET_RESET",
        "APPLET_CRYPTPW",
        "APPLET_MKPASSWD",
        "APPLET_VOLNAME",
        "APPLET_PIPE_PROGRESS",
        "APPLET_RUNLEVEL",
        "APPLET_RUN_PARTS",
        "APPLET_MORE",
        "APPLET_CATV",
        "INIT",
        "INIT_RUSTFS",
        "INIT_SYSVINIT",
        "INIT_SYSTEMD",
        "MDEV",
        "UDEV",
        "APPLET_CAT",
        "APPLET_CP",
        "APPLET_MV",
        "APPLET_RM",
        "APPLET_MKDIR",
        "APPLET_RMDIR",
        "APPLET_LS",
        "APPLET_TOUCH",
        "APPLET_LN",
        "APPLET_CHMOD",
        "APPLET_HEAD",
        "APPLET_TAIL",
        "APPLET_TEE",
        "APPLET_WC",
        "APPLET_DU",
        "APPLET_DF",
        "APPLET_STAT",
        "APPLET_READLINK",
        "APPLET_ECHO",
        "APPLET_PRINTF",
        "APPLET_GREP",
        "APPLET_SED",
        "APPLET_SORT",
        "APPLET_UNIQ",
        "APPLET_TR",
        "APPLET_CUT",
        "APPLET_PASTE",
        "APPLET_FOLD",
        "APPLET_REV",
        "APPLET_NL",
        "APPLET_BASENAME",
        "APPLET_DIRNAME",
        "APPLET_PWD",
        "APPLET_REALPATH",
        "APPLET_BASE64",
        "APPLET_MD5SUM",
        "APPLET_SHA256SUM",
        "APPLET_XXD",
        "APPLET_UNAME",
        "APPLET_HOSTNAME",
        "APPLET_WHOAMI",
        "APPLET_ID",
        "APPLET_UPTIME",
        "APPLET_DATE",
        "APPLET_ENV",
        "APPLET_PRINTENV",
        "APPLET_SLEEP",
        "APPLET_YES",
        "APPLET_TRUE",
        "APPLET_FALSE",
        "APPLET_NOHUP",
        "APPLET_SEQ",
        "APPLET_TTY",
        "APPLET_WHICH",
        "APPLET_XARGS",
        "APPLET_FIND",
        "APPLET_TEST",
        "APPLET_EXPR",
        "APPLET_ADDGROUP",
        "APPLET_ADDUSER",
        "APPLET_AR",
        "APPLET_ARP",
        "APPLET_ARPING",
        "APPLET_AWK",
        "APPLET_BLKID",
        "APPLET_BUNZIP2",
        "APPLET_BZCAT",
        "APPLET_BZIP2",
        "APPLET_CHGRP",
        "APPLET_CHOWN",
        "APPLET_CLEAR",
        "APPLET_DD",
        "APPLET_DELGROUP",
        "APPLET_DELUSER",
        "APPLET_DIFF",
        "APPLET_DMESG",
        "APPLET_DOS2UNIX",
        "APPLET_FBSET",
        "APPLET_FDISK",
        "APPLET_FSCK",
        "APPLET_FSYNC",
        "APPLET_FTPD",
        "APPLET_FTPGET",
        "APPLET_FTPPUT",
        "APPLET_FUSER",
        "APPLET_GETOPT",
        "APPLET_GETTY",
        "APPLET_GUNZIP",
        "APPLET_GZIP",
        "APPLET_HD",
        "APPLET_HEXDUMP",
        "APPLET_HOSTID",
        "APPLET_HTTPD",
        "APPLET_HWCLOCK",
        "APPLET_IFCONFIG",
        "APPLET_IFDOWN",
        "APPLET_IFUP",
        "APPLET_INSMOD",
        "APPLET_INSTALL",
        "APPLET_IP",
        "APPLET_IPADDR",
        "APPLET_IPCALC",
        "APPLET_IPCRM",
        "APPLET_IPCS",
        "APPLET_KILL",
        "APPLET_KILLALL",
        "APPLET_KLOGD",
        "APPLET_LAST",
        "APPLET_LENGTH",
        "APPLET_LESS",
        "APPLET_LOGGER",
        "APPLET_LOGIN",
        "APPLET_LOGNAME",
        "APPLET_LOGREAD",
        "APPLET_LOSETUP",
        "APPLET_LSMOD",
        "APPLET_MOUNT",
        "APPLET_UMOUNT",
        "APPLET_CHROOT",
        "APPLET_KEXEC",
        "APPLET_SWITCH_ROOT",
        "APPLET_RMMOD",
        "APPLET_DEPMOD",
        "APPLET_MODPROBE",
        "APPLET_MODINFO",
        "APPLET_SH",
        "APPLET_PS",
        "APPLET_FREE",
        "APPLET_SYNC",
        "APPLET_MKTEMP",
        "APPLET_NPROC",
        "APPLET_TAC",
        "APPLET_TIMEOUT",
        "APPLET_OD",
        "APPLET_TRUNCATE",
        "APPLET_STRINGS",
        "APPLET_CMP",
        "APPLET_COMM",
        "APPLET_CAL",
        "APPLET_CKSUM",
        "APPLET_SUM",
        "APPLET_EXPAND",
        "APPLET_UNEXPAND",
        "APPLET_SPLIT",
        "APPLET_UUENCODE",
        "APPLET_UUDECODE",
        "APPLET_UNIX2DOS",
        "APPLET_DNSDOMAINNAME",
        "APPLET_DC",
        "APPLET_SHA1SUM",
        "APPLET_SHA512SUM",
        "APPLET_PIDOF",
        "APPLET_PGREP",
        "APPLET_PKILL",
        "APPLET_KILLALL5",
        "APPLET_SETSID",
        "APPLET_USLEEP",
        "APPLET_NICE",
        "APPLET_RENICE",
        "APPLET_IONICE",
        "APPLET_CHRT",
        "APPLET_TASKSET",
        "APPLET_WHO",
        "APPLET_MESG",
        "APPLET_TTYSIZE",
        "APPLET_WATCH",
        "APPLET_TIME",
        "APPLET_MOUNTPOINT",
        "APPLET_PIVOT_ROOT",
        "APPLET_MKNOD",
        "APPLET_MKFIFO",
        "APPLET_DEVMEM",
        "APPLET_EJECT",
        "APPLET_FREERAMDISK",
        "APPLET_SWAPON",
        "APPLET_SWAPOFF",
        "APPLET_SYSCTL",
        "APPLET_FINDFS",
        "APPLET_MKSWAP",
        "APPLET_RDEV",
        "APPLET_LSATTR",
        "APPLET_CHATTR",
        "APPLET_FDFORMAT",
        "APPLET_HDPARM",
        "APPLET_FLASH_LOCK",
        "APPLET_FLASH_UNLOCK",
        "APPLET_READPROFILE",
        "APPLET_RTCWAKE",
        "APPLET_ADJTIMEX",
        "APPLET_RAIDAUTORUN",
        "APPLET_FDFLUSH",
    ];

    for opt in &all_options {
        let key = format!("CONFIG_{opt}");
        // Default to enabled if no .config file exists
        let enabled = configs.get(&key).map_or(!config_path.exists(), |v| v == "y");
        if enabled {
            // Emit lowercase cfg flag: e.g. cfg(applet_cat), cfg(init)
            println!("cargo:rustc-cfg={}", opt.to_lowercase());
        }
    }

    // Rebuild if .config changes
    println!("cargo:rerun-if-changed=.config");
    println!("cargo:rerun-if-changed=configs/default_defconfig");

    // musl has crypt() built into libc; only link libcrypt for glibc targets
    let target = std::env::var("TARGET").unwrap_or_default();
    if !target.contains("musl") {
        println!("cargo:rustc-link-lib=crypt");
    }
    println!("cargo:rerun-if-env-changed=TARGET");
}

fn parse_dotconfig(path: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
    }
    map
}
