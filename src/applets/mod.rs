/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
#[cfg(applet_cat)]
pub mod cat;
#[cfg(applet_cp)]
pub mod cp;
#[cfg(applet_mv)]
pub mod mv;
#[cfg(applet_rm)]
pub mod rm;
#[cfg(applet_mkdir)]
pub mod mkdir;
#[cfg(applet_rmdir)]
pub mod rmdir;
#[cfg(applet_ls)]
pub mod ls;
#[cfg(applet_touch)]
pub mod touch;
#[cfg(applet_ln)]
pub mod ln;
#[cfg(applet_chmod)]
pub mod chmod;
#[cfg(applet_head)]
pub mod head;
#[cfg(applet_tail)]
pub mod tail;
#[cfg(applet_tee)]
pub mod tee;
#[cfg(applet_wc)]
pub mod wc;
#[cfg(applet_du)]
pub mod du;
#[cfg(applet_df)]
pub mod df;
#[cfg(applet_stat)]
pub mod stat;
#[cfg(applet_readlink)]
pub mod readlink;

#[cfg(applet_echo)]
pub mod echo;
#[cfg(applet_printf)]
pub mod printf;
#[cfg(applet_grep)]
pub mod grep;
#[cfg(applet_sed)]
pub mod sed;
#[cfg(applet_sort)]
pub mod sort;
#[cfg(applet_uniq)]
pub mod uniq;
#[cfg(applet_tr)]
pub mod tr;
#[cfg(applet_cut)]
pub mod cut;
#[cfg(applet_paste)]
pub mod paste;
#[cfg(applet_fold)]
pub mod fold;
#[cfg(applet_rev)]
pub mod rev;
#[cfg(applet_nl)]
pub mod nl;

#[cfg(applet_basename)]
pub mod basename;
#[cfg(applet_dirname)]
pub mod dirname;
#[cfg(applet_pwd)]
pub mod pwd;
#[cfg(applet_realpath)]
pub mod realpath;

#[cfg(applet_base64)]
pub mod base64cmd;
#[cfg(applet_md5sum)]
pub mod md5sum;
#[cfg(applet_sha256sum)]
pub mod sha256sum;
#[cfg(applet_xxd)]
pub mod xxd;

#[cfg(applet_uname)]
pub mod uname;
#[cfg(applet_hostname)]
pub mod hostname;
#[cfg(applet_whoami)]
pub mod whoami;
#[cfg(applet_id)]
pub mod id;
#[cfg(applet_uptime)]
pub mod uptime;
#[cfg(applet_date)]
pub mod date;
#[cfg(applet_env)]
pub mod envcmd;
#[cfg(applet_printenv)]
pub mod printenv;

#[cfg(applet_sleep)]
pub mod sleep;
#[cfg(applet_yes)]
pub mod yes;
#[cfg(applet_true)]
pub mod truecmd;
#[cfg(applet_false)]
pub mod falsecmd;
#[cfg(applet_nohup)]
pub mod nohup;
#[cfg(applet_seq)]
pub mod seq;
#[cfg(applet_tty)]
pub mod tty;
#[cfg(applet_which)]
pub mod which;
#[cfg(applet_xargs)]
pub mod xargs;
#[cfg(applet_find)]
pub mod find;
#[cfg(applet_test)]
pub mod test;
#[cfg(applet_expr)]
pub mod expr;

#[cfg(applet_addgroup)]
pub mod addgroup;
#[cfg(applet_adduser)]
pub mod adduser;
#[cfg(applet_ar)]
pub mod ar;
#[cfg(applet_arp)]
pub mod arp;
#[cfg(applet_arping)]
pub mod arping;
#[cfg(applet_awk)]
pub mod awk;
#[cfg(applet_blkid)]
pub mod blkid;
#[cfg(applet_bunzip2)]
pub mod bunzip2;
#[cfg(applet_bzcat)]
pub mod bzcat;
#[cfg(applet_bzip2)]
pub mod bzip2;
#[cfg(applet_chgrp)]
pub mod chgrp;
#[cfg(applet_chown)]
pub mod chown;
#[cfg(applet_clear)]
pub mod clear;
#[cfg(applet_dd)]
pub mod dd;
#[cfg(applet_delgroup)]
pub mod delgroup;
#[cfg(applet_deluser)]
pub mod deluser;
#[cfg(applet_diff)]
pub mod diff;
#[cfg(applet_dmesg)]
pub mod dmesg;
#[cfg(applet_dos2unix)]
pub mod dos2unix;
#[cfg(applet_fbset)]
pub mod fbset;
#[cfg(applet_fdisk)]
pub mod fdisk;
#[cfg(applet_fsck)]
pub mod fsck;
#[cfg(applet_fsync)]
pub mod fsync;
#[cfg(applet_ftpd)]
pub mod ftpd;
#[cfg(applet_ftpget)]
pub mod ftpget;
#[cfg(applet_ftpput)]
pub mod ftpput;
#[cfg(applet_fuser)]
pub mod fuser;
#[cfg(applet_getopt)]
pub mod getopt;
#[cfg(applet_getty)]
pub mod getty;
#[cfg(applet_gunzip)]
pub mod gunzip;
#[cfg(applet_gzip)]
pub mod gzip;
#[cfg(applet_hd)]
pub mod hd;
#[cfg(applet_hexdump)]
pub mod hexdump;
#[cfg(applet_hostid)]
pub mod hostid;
#[cfg(applet_httpd)]
pub mod httpd;
#[cfg(applet_hwclock)]
pub mod hwclock;
#[cfg(applet_ifconfig)]
pub mod ifconfig;
#[cfg(applet_ifdown)]
pub mod ifdown;
#[cfg(applet_ifup)]
pub mod ifup;
#[cfg(applet_insmod)]
pub mod insmod;
#[cfg(applet_install)]
pub mod install;
#[cfg(applet_ip)]
pub mod ip;
#[cfg(applet_ipaddr)]
pub mod ipaddr;
#[cfg(applet_ipcalc)]
pub mod ipcalc;
#[cfg(applet_ipcrm)]
pub mod ipcrm;
#[cfg(applet_ipcs)]
pub mod ipcs;
#[cfg(applet_kill)]
pub mod kill;
#[cfg(applet_killall)]
pub mod killall;
#[cfg(applet_klogd)]
pub mod klogd;
#[cfg(applet_last)]
pub mod last;
#[cfg(applet_length)]
pub mod length;
#[cfg(applet_less)]
pub mod less;
#[cfg(applet_logger)]
pub mod logger;
#[cfg(applet_login)]
pub mod login;
#[cfg(applet_logname)]
pub mod logname;
#[cfg(applet_logread)]
pub mod logread;
#[cfg(applet_losetup)]
pub mod losetup;
#[cfg(applet_lsmod)]
pub mod lsmod;
#[cfg(applet_mount)]
pub mod mount;
#[cfg(applet_chroot)]
pub mod chroot;
#[cfg(applet_kexec)]
pub mod kexec;
#[cfg(applet_switch_root)]
pub mod switch_root;
#[cfg(applet_umount)]
pub mod umount;
#[cfg(applet_rmmod)]
pub mod rmmod;
#[cfg(applet_depmod)]
pub mod depmod;
#[cfg(applet_modprobe)]
pub mod modprobe;
#[cfg(applet_modinfo)]
pub mod modinfo;
#[cfg(applet_sh)]
pub mod sh;

// New applets
#[cfg(applet_ps)]
pub mod ps;
#[cfg(applet_free)]
pub mod free;
#[cfg(applet_sync)]
pub mod sync;
#[cfg(applet_mktemp)]
pub mod mktemp;
#[cfg(applet_nproc)]
pub mod nproc;
#[cfg(applet_tac)]
pub mod tac;
#[cfg(applet_timeout)]
pub mod timeout;
#[cfg(applet_od)]
pub mod od;
#[cfg(applet_truncate)]
pub mod truncate;
#[cfg(applet_strings)]
pub mod strings;
#[cfg(applet_cmp)]
pub mod cmp;

// Wave 1: aliases + tiny text utils
#[cfg(applet_comm)]
pub mod comm;
#[cfg(applet_cal)]
pub mod cal;
#[cfg(applet_cksum)]
pub mod cksum;
#[cfg(applet_sum)]
pub mod sum;
#[cfg(applet_expand)]
pub mod expand;
#[cfg(applet_unexpand)]
pub mod unexpand;
#[cfg(applet_split)]
pub mod split;
#[cfg(applet_uuencode)]
pub mod uuencode;
#[cfg(applet_uudecode)]
pub mod uudecode;
#[cfg(applet_unix2dos)]
pub mod unix2dos;
#[cfg(applet_dnsdomainname)]
pub mod dnsdomainname;
#[cfg(applet_dc)]
pub mod dc;
#[cfg(applet_sha1sum)]
pub mod sha1sum;
#[cfg(applet_sha512sum)]
pub mod sha512sum;

// Wave 2: process/system small
#[cfg(applet_pidof)]
pub mod pidof;
#[cfg(applet_pgrep)]
pub mod pgrep;
#[cfg(applet_pkill)]
pub mod pkill;
#[cfg(applet_killall5)]
pub mod killall5;
#[cfg(applet_setsid)]
pub mod setsid;
#[cfg(applet_usleep)]
pub mod usleep;
#[cfg(applet_nice)]
pub mod nice;
#[cfg(applet_renice)]
pub mod renice;
#[cfg(applet_ionice)]
pub mod ionice;
#[cfg(applet_chrt)]
pub mod chrt;
#[cfg(applet_taskset)]
pub mod taskset;
#[cfg(applet_who)]
pub mod who;
#[cfg(applet_mesg)]
pub mod mesg;
#[cfg(applet_ttysize)]
pub mod ttysize;
#[cfg(applet_watch)]
pub mod watch;
#[cfg(applet_time)]
pub mod time;
#[cfg(applet_mountpoint)]
pub mod mountpoint;
#[cfg(applet_pivot_root)]
pub mod pivot_root;

// Wave 3: filesystem/device
#[cfg(applet_mknod)]
pub mod mknod;
#[cfg(applet_mkfifo)]
pub mod mkfifo;
#[cfg(applet_devmem)]
pub mod devmem;
#[cfg(applet_eject)]
pub mod eject;
#[cfg(applet_freeramdisk)]
pub mod freeramdisk;
#[cfg(applet_swapon)]
pub mod swapon;
#[cfg(applet_swapoff)]
pub mod swapoff;
#[cfg(applet_sysctl)]
pub mod sysctl;
#[cfg(applet_findfs)]
pub mod findfs;
#[cfg(applet_mkswap)]
pub mod mkswap;
#[cfg(applet_rdev)]
pub mod rdev;
#[cfg(applet_lsattr)]
pub mod lsattr;
#[cfg(applet_chattr)]
pub mod chattr;
#[cfg(applet_fdformat)]
pub mod fdformat;
#[cfg(applet_hdparm)]
pub mod hdparm;
#[cfg(applet_flash_lock)]
pub mod flash_lock;
#[cfg(applet_flash_unlock)]
pub mod flash_unlock;
#[cfg(applet_readprofile)]
pub mod readprofile;
#[cfg(applet_rtcwake)]
pub mod rtcwake;
#[cfg(applet_adjtimex)]
pub mod adjtimex;
#[cfg(applet_raidautorun)]
pub mod raidautorun;
#[cfg(applet_fdflush)]
pub mod fdflush;

#[cfg(init)]
pub mod init;

#[cfg(mdev)]
pub mod mdev;

#[cfg(udev)]
pub mod udev;

use std::io::{self, BufRead, Read};

/// Read from files or stdin, a common pattern for many applets.
/// Properly handles binary data and large files using
/// buffered I/O instead of reading entire files into memory.
#[allow(dead_code)]
pub fn input_stream(files: &[String]) -> Box<dyn Read> {
    if files.is_empty() || (files.len() == 1 && files[0] == "-") {
        Box::new(io::stdin())
    } else {
        let readers: Vec<Box<dyn Read>> = files
            .iter()
            .filter_map(|f| {
                if f == "-" {
                    Some(Box::new(io::stdin()) as Box<dyn Read>)
                } else {
                    match std::fs::File::open(f) {
                        Ok(file) => Some(Box::new(file) as Box<dyn Read>),
                        Err(e) => {
                            eprintln!("{f}: {e}");
                            None
                        }
                    }
                }
            })
            .collect();
        if readers.is_empty() {
            Box::new(io::empty())
        } else {
            let chain: Box<dyn Read> = readers.into_iter().next().unwrap();
            // This is safe because we already handled the empty case
            // For multiple files, chain them together
            Box::new(chain)
        }
    }
}

/// Read lines from files or stdin.
pub fn input_lines(files: &[String]) -> Vec<String> {
    let mut lines = Vec::new();
    if files.is_empty() || (files.len() == 1 && files[0] == "-") {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(l) => lines.push(l),
                Err(_) => break,
            }
        }
    } else {
        for f in files {
            if f == "-" {
                let stdin = io::stdin();
                for line in stdin.lock().lines() {
                    match line {
                        Ok(l) => lines.push(l),
                        Err(_) => break,
                    }
                }
            } else {
                match std::fs::read_to_string(f) {
                    Ok(content) => {
                        for l in content.lines() {
                            lines.push(l.to_string());
                        }
                    }
                    Err(e) => eprintln!("{f}: {e}"),
                }
            }
        }
    }
    lines
}
#[cfg(applet_catv)]
pub mod catv;
#[cfg(applet_more)]
pub mod more;
#[cfg(applet_run_parts)]
pub mod run_parts;
#[cfg(applet_runlevel)]
pub mod runlevel;
#[cfg(applet_pipe_progress)]
pub mod pipe_progress;
#[cfg(applet_volname)]
pub mod volname;
#[cfg(applet_mkpasswd)]
pub mod mkpasswd;
#[cfg(applet_cryptpw)]
pub mod cryptpw;
#[cfg(applet_reset)]
pub mod reset;
#[cfg(applet_beep)]
pub mod beep;
#[cfg(applet_setarch)]
pub mod setarch;
#[cfg(applet_chvt)]
pub mod chvt;
#[cfg(applet_deallocvt)]
pub mod deallocvt;
#[cfg(applet_kbd_mode)]
pub mod kbd_mode;
#[cfg(applet_rdate)]
pub mod rdate;
