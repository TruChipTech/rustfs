/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! switch_root — free initramfs and switch to another root filesystem
//!
//! Usage: switch_root [-c /dev/console] NEW_ROOT INIT [ARGS...]
//!
//! Typically the last step of an initramfs: it recursively deletes the
//! contents of the current (initramfs) root, moves NEW_ROOT to /, chroots
//! into it, and execs INIT (e.g. /sbin/init) as the new PID 1.

use std::ffi::CString;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    let mut console: Option<String> = None;
    let mut positional: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--console" => {
                if let Some(c) = args.get(i + 1) {
                    console = Some(c.clone());
                    i += 2;
                    continue;
                }
                eprintln!("switch_root: option -c requires an argument");
                return 1;
            }
            "-h" | "--help" => {
                println!("Usage: switch_root [-c /dev/console] NEW_ROOT INIT [ARGS...]");
                return 0;
            }
            _ => positional.push(args[i].clone()),
        }
        i += 1;
    }

    if positional.len() < 2 {
        eprintln!("Usage: switch_root [-c /dev/console] NEW_ROOT INIT [ARGS...]");
        return 1;
    }

    let new_root = positional[0].clone();
    let init = positional[1].clone();
    let init_args = &positional[1..];

    // switch_root must run as PID 1.
    if std::process::id() != 1 {
        eprintln!("switch_root: must be run as PID 1");
        return 1;
    }

    // Sanity check: NEW_ROOT must be a mount point with an executable init.
    if !Path::new(&new_root).is_dir() {
        eprintln!("switch_root: bad newroot '{new_root}'");
        return 1;
    }
    let init_in_newroot = format!("{}/{}", new_root.trim_end_matches('/'), init.trim_start_matches('/'));
    if !Path::new(&init_in_newroot).exists() {
        eprintln!("switch_root: init '{init}' not found in '{new_root}'");
        return 1;
    }

    // Remember the device of the current root so we don't recurse onto other
    // filesystems while wiping the initramfs.
    let root_dev = match fs::metadata("/") {
        Ok(m) => m.dev(),
        Err(e) => {
            eprintln!("switch_root: stat /: {e}");
            return 1;
        }
    };

    // Recursively delete everything on the rootfs (initramfs) filesystem only.
    delete_recursive(Path::new("/"), root_dev);

    // Move the new root onto / and chroot into it.
    let c_newroot = CString::new(new_root.as_str()).unwrap();
    let c_slash = CString::new("/").unwrap();
    let c_dot = CString::new(".").unwrap();

    if unsafe { libc::chdir(c_newroot.as_ptr()) } != 0 {
        eprintln!("switch_root: chdir '{new_root}': {}", std::io::Error::last_os_error());
        return 1;
    }

    let ret = unsafe {
        libc::mount(
            c_dot.as_ptr(),
            c_slash.as_ptr(),
            std::ptr::null(),
            libc::MS_MOVE,
            std::ptr::null(),
        )
    };
    if ret != 0 {
        eprintln!("switch_root: mount --move '{new_root}' to /: {}", std::io::Error::last_os_error());
        return 1;
    }

    if unsafe { libc::chroot(c_dot.as_ptr()) } != 0 {
        eprintln!("switch_root: chroot: {}", std::io::Error::last_os_error());
        return 1;
    }
    if unsafe { libc::chdir(c_slash.as_ptr()) } != 0 {
        eprintln!("switch_root: chdir /: {}", std::io::Error::last_os_error());
        return 1;
    }

    // Reopen the console in the new root if requested.
    if let Some(con) = console {
        let c_con = CString::new(con.as_str()).unwrap();
        let fd = unsafe { libc::open(c_con.as_ptr(), libc::O_RDWR) };
        if fd >= 0 {
            unsafe {
                libc::dup2(fd, 0);
                libc::dup2(fd, 1);
                libc::dup2(fd, 2);
                if fd > 2 {
                    libc::close(fd);
                }
            }
        } else {
            eprintln!("switch_root: warning: cannot open console '{con}'");
        }
    }

    // exec the new init — replaces this process, keeping PID 1.
    let c_init = CString::new(init.as_str()).unwrap();
    let c_argv: Vec<CString> = init_args
        .iter()
        .map(|a| CString::new(a.as_str()).unwrap())
        .collect();
    let mut argv_ptrs: Vec<*const libc::c_char> = c_argv.iter().map(|a| a.as_ptr()).collect();
    argv_ptrs.push(std::ptr::null());

    unsafe { libc::execv(c_init.as_ptr(), argv_ptrs.as_ptr()) };

    // execv only returns on error.
    eprintln!("switch_root: exec '{init}': {}", std::io::Error::last_os_error());
    1
}

/// Recursively delete directory contents, staying on the filesystem identified
/// by `root_dev`. Never crosses mount points (e.g. the freshly mounted /proc,
/// /sys, /dev, or NEW_ROOT itself).
fn delete_recursive(dir: &Path, root_dev: u64) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if meta.is_dir() {
            // Don't descend into a different filesystem.
            if meta.dev() != root_dev {
                continue;
            }
            delete_recursive(&path, root_dev);
            let _ = fs::remove_dir(&path);
        } else {
            let _ = fs::remove_file(&path);
        }
    }
}
