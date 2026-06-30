/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! RustFS — Rust-based rootfs utilities
//!
//! Multi-call binary: behavior is determined by the name used to invoke the
//! binary (via symlinks) or by the first argument.

use std::env;
use std::path::Path;
use std::process;

mod applets;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Determine applet name: either from argv[0] (symlink name) or argv[1]
    let (applet_name, applet_args) = resolve_applet(&args);

    let exit_code = match applet_name.as_str() {
        // Init system
        #[cfg(init)]
        "init" => applets::init::run(&applet_args),

        // Device managers
        #[cfg(mdev)]
        "mdev" => applets::mdev::run(&applet_args),
        #[cfg(udev)]
        "udevd" | "udevadm" => applets::udev::run(&applet_args),

        // File utilities
        #[cfg(applet_cat)]
        "cat" => applets::cat::run(&applet_args),
        #[cfg(applet_cp)]
        "cp" => applets::cp::run(&applet_args),
        #[cfg(applet_mv)]
        "mv" => applets::mv::run(&applet_args),
        #[cfg(applet_rm)]
        "rm" => applets::rm::run(&applet_args),
        #[cfg(applet_mkdir)]
        "mkdir" => applets::mkdir::run(&applet_args),
        #[cfg(applet_rmdir)]
        "rmdir" => applets::rmdir::run(&applet_args),
        #[cfg(applet_ls)]
        "ls" => applets::ls::run(&applet_args),
        #[cfg(applet_touch)]
        "touch" => applets::touch::run(&applet_args),
        #[cfg(applet_ln)]
        "ln" => applets::ln::run(&applet_args),
        #[cfg(applet_chmod)]
        "chmod" => applets::chmod::run(&applet_args),
        #[cfg(applet_head)]
        "head" => applets::head::run(&applet_args),
        #[cfg(applet_tail)]
        "tail" => applets::tail::run(&applet_args),
        #[cfg(applet_tee)]
        "tee" => applets::tee::run(&applet_args),
        #[cfg(applet_wc)]
        "wc" => applets::wc::run(&applet_args),
        #[cfg(applet_du)]
        "du" => applets::du::run(&applet_args),
        #[cfg(applet_df)]
        "df" => applets::df::run(&applet_args),
        #[cfg(applet_stat)]
        "stat" => applets::stat::run(&applet_args),
        #[cfg(applet_readlink)]
        "readlink" => applets::readlink::run(&applet_args),

        // Text processing
        #[cfg(applet_echo)]
        "echo" => applets::echo::run(&applet_args),
        #[cfg(applet_printf)]
        "printf" => applets::printf::run(&applet_args),
        #[cfg(applet_grep)]
        "grep" => applets::grep::run(&applet_args),
        #[cfg(applet_sed)]
        "sed" => applets::sed::run(&applet_args),
        #[cfg(applet_sort)]
        "sort" => applets::sort::run(&applet_args),
        #[cfg(applet_uniq)]
        "uniq" => applets::uniq::run(&applet_args),
        #[cfg(applet_tr)]
        "tr" => applets::tr::run(&applet_args),
        #[cfg(applet_cut)]
        "cut" => applets::cut::run(&applet_args),
        #[cfg(applet_paste)]
        "paste" => applets::paste::run(&applet_args),
        #[cfg(applet_fold)]
        "fold" => applets::fold::run(&applet_args),
        #[cfg(applet_rev)]
        "rev" => applets::rev::run(&applet_args),
        #[cfg(applet_nl)]
        "nl" => applets::nl::run(&applet_args),

        // Path / string utilities
        #[cfg(applet_basename)]
        "basename" => applets::basename::run(&applet_args),
        #[cfg(applet_dirname)]
        "dirname" => applets::dirname::run(&applet_args),
        #[cfg(applet_pwd)]
        "pwd" => applets::pwd::run(&applet_args),
        #[cfg(applet_realpath)]
        "realpath" => applets::realpath::run(&applet_args),

        // Encoding / hashing
        #[cfg(applet_base64)]
        "base64" => applets::base64cmd::run(&applet_args),
        #[cfg(applet_md5sum)]
        "md5sum" => applets::md5sum::run(&applet_args),
        #[cfg(applet_sha256sum)]
        "sha256sum" => applets::sha256sum::run(&applet_args),
        #[cfg(applet_xxd)]
        "xxd" => applets::xxd::run(&applet_args),

        // System info
        #[cfg(applet_uname)]
        "uname" => applets::uname::run(&applet_args),
        #[cfg(applet_hostname)]
        "hostname" => applets::hostname::run(&applet_args),
        #[cfg(applet_whoami)]
        "whoami" => applets::whoami::run(&applet_args),
        #[cfg(applet_id)]
        "id" => applets::id::run(&applet_args),
        #[cfg(applet_uptime)]
        "uptime" => applets::uptime::run(&applet_args),
        #[cfg(applet_date)]
        "date" => applets::date::run(&applet_args),
        #[cfg(applet_env)]
        "env" => applets::envcmd::run(&applet_args),
        #[cfg(applet_printenv)]
        "printenv" => applets::printenv::run(&applet_args),

        // Process utilities
        #[cfg(applet_sleep)]
        "sleep" => applets::sleep::run(&applet_args),
        #[cfg(applet_yes)]
        "yes" => applets::yes::run(&applet_args),
        #[cfg(applet_true)]
        "true" => applets::truecmd::run(&applet_args),
        #[cfg(applet_false)]
        "false" => applets::falsecmd::run(&applet_args),
        #[cfg(applet_nohup)]
        "nohup" => applets::nohup::run(&applet_args),
        #[cfg(applet_seq)]
        "seq" => applets::seq::run(&applet_args),
        #[cfg(applet_tty)]
        "tty" => applets::tty::run(&applet_args),
        #[cfg(applet_which)]
        "which" => applets::which::run(&applet_args),
        #[cfg(applet_xargs)]
        "xargs" => applets::xargs::run(&applet_args),
        #[cfg(applet_find)]
        "find" => applets::find::run(&applet_args),
        #[cfg(applet_test)]
        "test" | "[" => applets::test::run(&applet_args),
        #[cfg(applet_expr)]
        "expr" => applets::expr::run(&applet_args),

        // User/group management
        #[cfg(applet_addgroup)]
        "addgroup" => applets::addgroup::run(&applet_args),
        #[cfg(applet_adduser)]
        "adduser" => applets::adduser::run(&applet_args),
        #[cfg(applet_delgroup)]
        "delgroup" => applets::delgroup::run(&applet_args),
        #[cfg(applet_deluser)]
        "deluser" => applets::deluser::run(&applet_args),
        #[cfg(applet_chgrp)]
        "chgrp" => applets::chgrp::run(&applet_args),
        #[cfg(applet_chown)]
        "chown" => applets::chown::run(&applet_args),

        // Archive / compression
        #[cfg(applet_ar)]
        "ar" => applets::ar::run(&applet_args),
        #[cfg(applet_bunzip2)]
        "bunzip2" => applets::bunzip2::run(&applet_args),
        #[cfg(applet_bzcat)]
        "bzcat" => applets::bzcat::run(&applet_args),
        #[cfg(applet_bzip2)]
        "bzip2" => applets::bzip2::run(&applet_args),

        // Networking
        #[cfg(applet_arp)]
        "arp" => applets::arp::run(&applet_args),
        #[cfg(applet_arping)]
        "arping" => applets::arping::run(&applet_args),
        #[cfg(applet_ftpd)]
        "ftpd" => applets::ftpd::run(&applet_args),
        #[cfg(applet_ftpget)]
        "ftpget" => applets::ftpget::run(&applet_args),
        #[cfg(applet_ftpput)]
        "ftpput" => applets::ftpput::run(&applet_args),
        #[cfg(applet_httpd)]
        "httpd" => applets::httpd::run(&applet_args),
        #[cfg(applet_ifconfig)]
        "ifconfig" => applets::ifconfig::run(&applet_args),
        #[cfg(applet_ifdown)]
        "ifdown" => applets::ifdown::run(&applet_args),
        #[cfg(applet_ifup)]
        "ifup" => applets::ifup::run(&applet_args),
        #[cfg(applet_ip)]
        "ip" => applets::ip::run(&applet_args),
        #[cfg(applet_ipaddr)]
        "ipaddr" => applets::ipaddr::run(&applet_args),
        #[cfg(applet_ipcalc)]
        "ipcalc" => applets::ipcalc::run(&applet_args),

        // Text / data processing
        #[cfg(applet_awk)]
        "awk" => applets::awk::run(&applet_args),
        #[cfg(applet_diff)]
        "diff" => applets::diff::run(&applet_args),
        #[cfg(applet_dos2unix)]
        "dos2unix" => applets::dos2unix::run(&applet_args),
        #[cfg(applet_dd)]
        "dd" => applets::dd::run(&applet_args),
        #[cfg(applet_getopt)]
        "getopt" => applets::getopt::run(&applet_args),
        #[cfg(applet_hd)]
        "hd" => applets::hd::run(&applet_args),
        #[cfg(applet_hexdump)]
        "hexdump" => applets::hexdump::run(&applet_args),
        #[cfg(applet_length)]
        "length" => applets::length::run(&applet_args),
        #[cfg(applet_less)]
        "less" => applets::less::run(&applet_args),

        // System / disk utilities
        #[cfg(applet_blkid)]
        "blkid" => applets::blkid::run(&applet_args),
        #[cfg(applet_clear)]
        "clear" => applets::clear::run(&applet_args),
        #[cfg(applet_dmesg)]
        "dmesg" => applets::dmesg::run(&applet_args),
        #[cfg(applet_fbset)]
        "fbset" => applets::fbset::run(&applet_args),
        #[cfg(applet_fdisk)]
        "fdisk" => applets::fdisk::run(&applet_args),
        #[cfg(applet_fsck)]
        "fsck" => applets::fsck::run(&applet_args),
        #[cfg(applet_fsync)]
        "fsync" => applets::fsync::run(&applet_args),
        #[cfg(applet_hwclock)]
        "hwclock" => applets::hwclock::run(&applet_args),
        #[cfg(applet_insmod)]
        "insmod" => applets::insmod::run(&applet_args),
        #[cfg(applet_install)]
        "install" => applets::install::run(&applet_args),
        #[cfg(applet_klogd)]
        "klogd" => applets::klogd::run(&applet_args),
        #[cfg(applet_losetup)]
        "losetup" => applets::losetup::run(&applet_args),
        #[cfg(applet_lsmod)]
        "lsmod" => applets::lsmod::run(&applet_args),
        #[cfg(applet_mount)]
        "mount" => applets::mount::run(&applet_args),
        #[cfg(applet_umount)]
        "umount" => applets::umount::run(&applet_args),
        #[cfg(applet_chroot)]
        "chroot" => applets::chroot::run(&applet_args),
        #[cfg(applet_kexec)]
        "kexec" => applets::kexec::run(&applet_args),
        #[cfg(applet_switch_root)]
        "switch_root" => applets::switch_root::run(&applet_args),
        #[cfg(applet_rmmod)]
        "rmmod" => applets::rmmod::run(&applet_args),
        #[cfg(applet_depmod)]
        "depmod" => applets::depmod::run(&applet_args),
        #[cfg(applet_modprobe)]
        "modprobe" => applets::modprobe::run(&applet_args),
        #[cfg(applet_modinfo)]
        "modinfo" => applets::modinfo::run(&applet_args),

        // Process utilities
        #[cfg(applet_fuser)]
        "fuser" => applets::fuser::run(&applet_args),
        #[cfg(applet_getty)]
        "getty" => applets::getty::run(&applet_args),
        #[cfg(applet_kill)]
        "kill" => applets::kill::run(&applet_args),
        #[cfg(applet_killall)]
        "killall" => applets::killall::run(&applet_args),
        #[cfg(applet_login)]
        "login" => applets::login::run(&applet_args),

        // Archive / compression
        #[cfg(applet_gunzip)]
        "gunzip" => applets::gunzip::run(&applet_args),
        #[cfg(applet_gzip)]
        "gzip" => applets::gzip::run(&applet_args),

        // Logging
        #[cfg(applet_logger)]
        "logger" => applets::logger::run(&applet_args),
        #[cfg(applet_logread)]
        "logread" => applets::logread::run(&applet_args),

        // System info (additional)
        #[cfg(applet_hostid)]
        "hostid" => applets::hostid::run(&applet_args),
        #[cfg(applet_logname)]
        "logname" => applets::logname::run(&applet_args),
        #[cfg(applet_last)]
        "last" => applets::last::run(&applet_args),

        // IPC utilities
        #[cfg(applet_ipcrm)]
        "ipcrm" => applets::ipcrm::run(&applet_args),
        #[cfg(applet_ipcs)]
        "ipcs" => applets::ipcs::run(&applet_args),

        // Shell
        #[cfg(applet_sh)]
        "sh" | "ash" => applets::sh::run(&applet_args),

        // New applets (BusyBox parity)
        #[cfg(applet_ps)]
        "ps" => applets::ps::run(&applet_args),
        #[cfg(applet_free)]
        "free" => applets::free::run(&applet_args),
        #[cfg(applet_sync)]
        "sync" => applets::sync::run(&applet_args),
        #[cfg(applet_mktemp)]
        "mktemp" => applets::mktemp::run(&applet_args),
        #[cfg(applet_nproc)]
        "nproc" => applets::nproc::run(&applet_args),
        #[cfg(applet_tac)]
        "tac" => applets::tac::run(&applet_args),
        #[cfg(applet_timeout)]
        "timeout" => applets::timeout::run(&applet_args),
        #[cfg(applet_od)]
        "od" => applets::od::run(&applet_args),
        #[cfg(applet_truncate)]
        "truncate" => applets::truncate::run(&applet_args),
        #[cfg(applet_strings)]
        "strings" => applets::strings::run(&applet_args),
        #[cfg(applet_cmp)]
        "cmp" => applets::cmp::run(&applet_args),

        // Wave 1: aliases + tiny text utils
        #[cfg(applet_grep)]
        "egrep" => {
            let mut a = vec!["-E".to_string()];
            a.extend(applet_args.iter().cloned());
            applets::grep::run(&a)
        }
        #[cfg(applet_grep)]
        "fgrep" => {
            let mut a = vec!["-F".to_string()];
            a.extend(applet_args.iter().cloned());
            applets::grep::run(&a)
        }
        #[cfg(applet_gunzip)]
        "zcat" => {
            let mut a = vec!["-c".to_string()];
            a.extend(applet_args.iter().cloned());
            applets::gunzip::run(&a)
        }
        #[cfg(applet_comm)]
        "comm" => applets::comm::run(&applet_args),
        #[cfg(applet_cal)]
        "cal" => applets::cal::run(&applet_args),
        #[cfg(applet_cksum)]
        "cksum" => applets::cksum::run(&applet_args),
        #[cfg(applet_sum)]
        "sum" => applets::sum::run(&applet_args),
        #[cfg(applet_expand)]
        "expand" => applets::expand::run(&applet_args),
        #[cfg(applet_unexpand)]
        "unexpand" => applets::unexpand::run(&applet_args),
        #[cfg(applet_split)]
        "split" => applets::split::run(&applet_args),
        #[cfg(applet_uuencode)]
        "uuencode" => applets::uuencode::run(&applet_args),
        #[cfg(applet_uudecode)]
        "uudecode" => applets::uudecode::run(&applet_args),
        #[cfg(applet_unix2dos)]
        "unix2dos" => applets::unix2dos::run(&applet_args),
        #[cfg(applet_dnsdomainname)]
        "dnsdomainname" => applets::dnsdomainname::run(&applet_args),
        #[cfg(applet_dc)]
        "dc" => applets::dc::run(&applet_args),
        #[cfg(applet_sha1sum)]
        "sha1sum" => applets::sha1sum::run(&applet_args),
        #[cfg(applet_sha512sum)]
        "sha512sum" => applets::sha512sum::run(&applet_args),

        // Wave 2: process/system small
        #[cfg(applet_pidof)]
        "pidof" => applets::pidof::run(&applet_args),
        #[cfg(applet_pgrep)]
        "pgrep" => applets::pgrep::run(&applet_args),
        #[cfg(applet_pkill)]
        "pkill" => applets::pkill::run(&applet_args),
        #[cfg(applet_killall5)]
        "killall5" => applets::killall5::run(&applet_args),
        #[cfg(applet_setsid)]
        "setsid" => applets::setsid::run(&applet_args),
        #[cfg(applet_usleep)]
        "usleep" => applets::usleep::run(&applet_args),
        #[cfg(applet_nice)]
        "nice" => applets::nice::run(&applet_args),
        #[cfg(applet_renice)]
        "renice" => applets::renice::run(&applet_args),
        #[cfg(applet_ionice)]
        "ionice" => applets::ionice::run(&applet_args),
        #[cfg(applet_chrt)]
        "chrt" => applets::chrt::run(&applet_args),
        #[cfg(applet_taskset)]
        "taskset" => applets::taskset::run(&applet_args),
        #[cfg(applet_who)]
        "who" => applets::who::run(&applet_args),
        #[cfg(applet_mesg)]
        "mesg" => applets::mesg::run(&applet_args),
        #[cfg(applet_ttysize)]
        "ttysize" => applets::ttysize::run(&applet_args),
        #[cfg(applet_watch)]
        "watch" => applets::watch::run(&applet_args),
        #[cfg(applet_time)]
        "time" => applets::time::run(&applet_args),
        #[cfg(applet_mountpoint)]
        "mountpoint" => applets::mountpoint::run(&applet_args),
        #[cfg(applet_pivot_root)]
        "pivot_root" => applets::pivot_root::run(&applet_args),

        // Wave 3: filesystem/device
        #[cfg(applet_mknod)]
        "mknod" => applets::mknod::run(&applet_args),
        #[cfg(applet_mkfifo)]
        "mkfifo" => applets::mkfifo::run(&applet_args),
        #[cfg(applet_devmem)]
        "devmem" => applets::devmem::run(&applet_args),
        #[cfg(applet_eject)]
        "eject" => applets::eject::run(&applet_args),
        #[cfg(applet_freeramdisk)]
        "freeramdisk" => applets::freeramdisk::run(&applet_args),
        #[cfg(applet_swapon)]
        "swapon" => applets::swapon::run(&applet_args),
        #[cfg(applet_swapoff)]
        "swapoff" => applets::swapoff::run(&applet_args),
        #[cfg(applet_sysctl)]
        "sysctl" => applets::sysctl::run(&applet_args),
        #[cfg(applet_findfs)]
        "findfs" => applets::findfs::run(&applet_args),
        #[cfg(applet_mkswap)]
        "mkswap" => applets::mkswap::run(&applet_args),
        #[cfg(applet_rdev)]
        "rdev" => applets::rdev::run(&applet_args),
        #[cfg(applet_lsattr)]
        "lsattr" => applets::lsattr::run(&applet_args),
        #[cfg(applet_chattr)]
        "chattr" => applets::chattr::run(&applet_args),
        #[cfg(applet_fdformat)]
        "fdformat" => applets::fdformat::run(&applet_args),
        #[cfg(applet_hdparm)]
        "hdparm" => applets::hdparm::run(&applet_args),
        #[cfg(applet_flash_lock)]
        "flash_lock" => applets::flash_lock::run(&applet_args),
        #[cfg(applet_flash_unlock)]
        "flash_unlock" => applets::flash_unlock::run(&applet_args),
        #[cfg(applet_readprofile)]
        "readprofile" => applets::readprofile::run(&applet_args),
        #[cfg(applet_rtcwake)]
        "rtcwake" => applets::rtcwake::run(&applet_args),
        #[cfg(applet_adjtimex)]
        "adjtimex" => applets::adjtimex::run(&applet_args),
        #[cfg(applet_raidautorun)]
        "raidautorun" => applets::raidautorun::run(&applet_args),
        #[cfg(applet_fdflush)]
        "fdflush" => applets::fdflush::run(&applet_args),

        // Help / meta
        "help" | "--help" | "-h" => {
            print_help();
            0
        }
        "rustfs" => {
            if applet_args.is_empty() {
                print_help();
                0
            } else {
                eprintln!("rustfs: unknown applet '{}'", applet_args[0]);
                1
            }
        }
        _ => {
            eprintln!("rustfs: unknown applet '{}'. Run 'rustfs --help' for a list.", applet_name);
            1
        }
    };

    process::exit(exit_code);
}

/// Resolve the applet name from argv[0] (symlink) or argv[1] (subcommand).
fn resolve_applet(args: &[String]) -> (String, Vec<String>) {
    let binary = Path::new(&args[0])
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("rustfs")
        .to_lowercase();

    // Strip leading '-' for login shell convention (e.g. "-sh" → "sh")
    let binary = binary.strip_prefix('-').unwrap_or(&binary).to_string();

    if binary == "rustfs" {
        // Subcommand mode: rustfs cat file.txt
        if args.len() > 1 {
            let applet = args[1].clone();
            let rest = args[2..].to_vec();
            (applet, rest)
        } else {
            ("rustfs".to_string(), vec![])
        }
    } else {
        // Symlink mode: cat file.txt  (where cat -> rustfs)
        let rest = args[1..].to_vec();
        (binary, rest)
    }
}

fn print_help() {
    let version = env!("CARGO_PKG_VERSION");
    println!("RustFS v{version} — Rust-based rootfs utilities");
    println!();
    println!("Usage: rustfs <applet> [arguments...]");
    println!("   or: <applet> [arguments...]  (via symlink)");
    println!();
    println!("Available applets:");
    println!();
    let applets = [
        "addgroup", "adduser", "ar", "arp", "arping", "awk", "base64", "basename",
        "blkid", "bunzip2", "bzcat", "bzip2", "cat", "chgrp", "chmod", "chown",
        "clear", "cmp", "cp", "cut", "date", "dd", "delgroup", "deluser", "df",
        "diff", "dirname", "dmesg", "dos2unix", "du", "echo", "env", "expr",
        "false", "fbset", "fdisk", "find", "fold", "free", "fsck", "fsync",
        "ftpd", "ftpget", "ftpput", "fuser", "getopt", "getty", "grep", "gunzip",
        "gzip", "hd", "head", "hexdump", "hostid", "hostname", "httpd", "hwclock",
        "id", "ifconfig", "ifdown", "ifup", "init", "insmod", "install", "ip",
        "ipaddr", "ipcalc", "ipcrm", "ipcs", "kill", "killall", "klogd", "last",
        "length", "less", "ln", "logger", "login", "logname", "logread", "losetup",
        "ls", "lsmod", "md5sum", "mdev", "mkdir", "mktemp", "mv", "nl", "nohup",
        "nproc", "od", "paste", "printenv", "printf", "ps", "pwd", "readlink",
        "realpath", "rev", "rm", "rmdir", "sed", "seq", "sha256sum", "sleep",
        "chroot", "kexec", "switch_root",
        "sort", "stat", "strings", "sync", "tac", "tail", "tee", "test", "timeout",
        "touch", "tr", "true", "truncate", "tty", "udevadm", "udevd", "uname",
        "uniq", "uptime", "wc", "which", "whoami", "xargs", "xxd", "yes",
        "egrep", "fgrep", "zcat", "comm", "cal", "cksum", "sum", "expand",
        "unexpand", "split", "uuencode", "uudecode", "unix2dos", "dnsdomainname",
        "dc", "sha1sum", "sha512sum",
        "pidof", "pgrep", "pkill", "killall5", "setsid", "usleep", "nice",
        "renice", "ionice", "chrt", "taskset", "who", "mesg", "ttysize",
        "watch", "time", "mountpoint", "pivot_root",
        "mknod", "mkfifo", "devmem", "eject", "freeramdisk", "swapon", "swapoff",
        "sysctl", "findfs", "mkswap", "rdev", "lsattr", "chattr", "fdformat",
        "hdparm", "flash_lock", "flash_unlock", "readprofile", "rtcwake",
        "adjtimex", "raidautorun", "fdflush",
    ];
    for line in applets.chunks(10) {
        println!("  {}", line.join(", "));
    }
    println!();
}
