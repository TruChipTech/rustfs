/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! umount — unmount a filesystem
//!
//! Usage: umount [-f] [-l] [-r] [-d] [-a [-t type]] [target...]

use std::ffi::CString;
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut force = false;
    let mut lazy = false;
    let mut remount_ro = false;
    let mut unmount_all = false;
    let mut fstype_filter: Option<String> = None;
    let mut targets: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-f" | "--force" => force = true,
            "-l" | "--lazy" => lazy = true,
            "-r" => remount_ro = true,
            "-d" => {} // detach loopback (ignored)
            "-n" => {} // no mtab (ignored)
            "-a" | "--all" => unmount_all = true,
            "-t" => {
                if let Some(t) = args.get(i + 1) {
                    fstype_filter = Some(t.clone());
                    i += 2;
                    continue;
                } else {
                    eprintln!("umount: option -t requires an argument");
                    return 1;
                }
            }
            "-h" | "--help" => {
                println!("Usage: umount [-f] [-l] [-r] [-a [-t type]] [target...]");
                return 0;
            }
            _ => targets.push(args[i].clone()),
        }
        i += 1;
    }

    if unmount_all {
        return unmount_all_fs(fstype_filter.as_deref(), force, lazy, remount_ro);
    }

    if targets.is_empty() {
        eprintln!("umount: missing operand");
        return 1;
    }

    let mut ret = 0;
    for target in &targets {
        if do_umount(target, force, lazy, remount_ro) != 0 {
            ret = 1;
        }
    }
    ret
}

/// Unmount all mounted filesystems (except essential ones)
fn unmount_all_fs(
    fstype_filter: Option<&str>,
    force: bool,
    lazy: bool,
    remount_ro: bool,
) -> i32 {
    let content = match fs::read_to_string("/proc/mounts") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("umount: /proc/mounts: {e}");
            return 1;
        }
    };

    // Collect mounts in reverse order (unmount leaf mounts first)
    let mut mounts: Vec<(&str, &str)> = Vec::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let mountpoint = parts[1];
        let fstype = parts[2];

        // Skip essential pseudo-filesystems
        if matches!(mountpoint, "/" | "/proc" | "/sys" | "/dev") {
            continue;
        }

        if let Some(ft) = fstype_filter {
            if fstype != ft {
                continue;
            }
        }

        mounts.push((mountpoint, fstype));
    }

    mounts.reverse();

    let mut ret = 0;
    for (mountpoint, _) in &mounts {
        if do_umount(mountpoint, force, lazy, remount_ro) != 0 {
            ret = 1;
        }
    }
    ret
}

/// Perform the actual umount
fn do_umount(target: &str, force: bool, lazy: bool, remount_ro: bool) -> i32 {
    let c_target = CString::new(target).unwrap();

    let mut flags: libc::c_int = 0;
    if force {
        flags |= libc::MNT_FORCE;
    }
    if lazy {
        flags |= libc::MNT_DETACH;
    }

    let ret = unsafe { libc::umount2(c_target.as_ptr(), flags) };

    if ret != 0 {
        let err = std::io::Error::last_os_error();
        if remount_ro {
            // Try remounting read-only instead
            let c_none = CString::new("none").unwrap();
            let ro_ret = unsafe {
                libc::mount(
                    std::ptr::null(),
                    c_target.as_ptr(),
                    std::ptr::null(),
                    libc::MS_REMOUNT | libc::MS_RDONLY,
                    std::ptr::null(),
                )
            };
            if ro_ret == 0 {
                let _ = c_none;
                return 0;
            }
        }
        eprintln!("umount: {target}: {err}");
        return 1;
    }

    0
}
