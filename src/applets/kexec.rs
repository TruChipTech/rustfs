/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! kexec — load and boot into a new kernel without a full reboot
//!
//! Usage: kexec -l KERNEL [--initrd=FILE] [--append=CMDLINE]
//!        kexec -u                 (unload the currently loaded kernel)
//!        kexec -e                 (boot into the loaded kernel)
//!        kexec [-f] KERNEL ...    (load and boot in one step)
//!
//! Loading uses the kexec_file_load(2) syscall, which lets the kernel read
//! and (where required) verify the images itself. Booting is done via
//! reboot(LINUX_REBOOT_CMD_KEXEC).

use std::ffi::CString;
use std::os::unix::io::AsRawFd;

// Flags for kexec_file_load(2).
const KEXEC_FILE_UNLOAD: libc::c_ulong = 0x1;
const KEXEC_FILE_NO_INITRAMFS: libc::c_ulong = 0x4;

// The kexec_file_load(2) syscall number is part of the per-arch kernel ABI.
// libc does not expose `SYS_kexec_file_load` for every target (notably musl
// aarch64), so define it directly for the architectures we support.
#[cfg(target_arch = "x86_64")]
const SYS_KEXEC_FILE_LOAD: libc::c_long = 320;
#[cfg(target_arch = "aarch64")]
const SYS_KEXEC_FILE_LOAD: libc::c_long = 294;
#[cfg(target_arch = "arm")]
const SYS_KEXEC_FILE_LOAD: libc::c_long = 401;
// riscv64 follows the asm-generic syscall table (kexec_file_load = 294).
#[cfg(target_arch = "riscv64")]
const SYS_KEXEC_FILE_LOAD: libc::c_long = 294;
// Fall back to the libc-provided constant on any other architecture.
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "arm", target_arch = "riscv64")))]
const SYS_KEXEC_FILE_LOAD: libc::c_long = libc::SYS_kexec_file_load;

pub fn run(args: &[String]) -> i32 {
    let mut load = false;
    let mut unload = false;
    let mut exec = false;
    let mut force = false;
    let mut initrd: Option<String> = None;
    let mut cmdline = String::new();
    let mut kernel: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-l" | "--load" => load = true,
            "-u" | "--unload" => unload = true,
            "-e" | "--exec" => exec = true,
            "-f" | "--force" => force = true,
            "--initrd" | "-i" => {
                if let Some(v) = args.get(i + 1) {
                    initrd = Some(v.clone());
                    i += 2;
                    continue;
                }
                eprintln!("kexec: option --initrd requires an argument");
                return 1;
            }
            "--append" | "-c" => {
                if let Some(v) = args.get(i + 1) {
                    cmdline = v.clone();
                    i += 2;
                    continue;
                }
                eprintln!("kexec: option --append requires an argument");
                return 1;
            }
            "-h" | "--help" => {
                print_usage();
                return 0;
            }
            _ => {
                if let Some(v) = arg.strip_prefix("--initrd=") {
                    initrd = Some(v.to_string());
                } else if let Some(v) = arg.strip_prefix("--append=") {
                    cmdline = v.to_string();
                } else if arg.starts_with('-') {
                    eprintln!("kexec: unknown option '{arg}'");
                    return 1;
                } else {
                    kernel = Some(arg.clone());
                }
            }
        }
        i += 1;
    }

    // -f implies load-and-execute.
    if force {
        load = true;
        exec = true;
    }

    if unload {
        return do_unload();
    }

    if load {
        let kernel = match kernel {
            Some(k) => k,
            None => {
                eprintln!("kexec: no kernel image specified");
                return 1;
            }
        };
        if do_load(&kernel, initrd.as_deref(), &cmdline) != 0 {
            return 1;
        }
    }

    if exec {
        return do_exec();
    }

    if !load {
        print_usage();
        return 1;
    }

    0
}

fn do_load(kernel: &str, initrd: Option<&str>, cmdline: &str) -> i32 {
    let kernel_file = match std::fs::File::open(kernel) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("kexec: cannot open kernel '{kernel}': {e}");
            return 1;
        }
    };
    let kernel_fd = kernel_file.as_raw_fd();

    let mut flags: libc::c_ulong = 0;
    let initrd_file = match initrd {
        Some(path) => match std::fs::File::open(path) {
            Ok(f) => Some(f),
            Err(e) => {
                eprintln!("kexec: cannot open initrd '{path}': {e}");
                return 1;
            }
        },
        None => {
            flags |= KEXEC_FILE_NO_INITRAMFS;
            None
        }
    };
    let initrd_fd = initrd_file.as_ref().map_or(-1, |f| f.as_raw_fd());

    // The cmdline passed to the kernel must be NUL-terminated and its length
    // counted including the terminator.
    let c_cmdline = CString::new(cmdline).unwrap();
    let cmdline_bytes = c_cmdline.as_bytes_with_nul();

    let ret = unsafe {
        libc::syscall(
            SYS_KEXEC_FILE_LOAD,
            kernel_fd as libc::c_int,
            initrd_fd as libc::c_int,
            cmdline_bytes.len() as libc::c_ulong,
            cmdline_bytes.as_ptr(),
            flags,
        )
    };

    if ret != 0 {
        eprintln!("kexec: kexec_file_load: {}", std::io::Error::last_os_error());
        return 1;
    }
    0
}

fn do_unload() -> i32 {
    let ret = unsafe {
        libc::syscall(
            SYS_KEXEC_FILE_LOAD,
            -1i32 as libc::c_int,
            -1i32 as libc::c_int,
            0 as libc::c_ulong,
            std::ptr::null::<libc::c_char>(),
            KEXEC_FILE_UNLOAD,
        )
    };
    if ret != 0 {
        eprintln!("kexec: unload: {}", std::io::Error::last_os_error());
        return 1;
    }
    0
}

fn do_exec() -> i32 {
    // Flush filesystems before jumping into the new kernel.
    unsafe { libc::sync() };

    let ret = unsafe { libc::reboot(libc::LINUX_REBOOT_CMD_KEXEC) };
    // reboot() only returns on failure.
    eprintln!("kexec: reboot(KEXEC): {}", std::io::Error::last_os_error());
    let _ = ret;
    1
}

fn print_usage() {
    println!("Usage: kexec -l KERNEL [--initrd=FILE] [--append=CMDLINE]");
    println!("       kexec -u                 unload loaded kernel");
    println!("       kexec -e                 boot into loaded kernel");
    println!("       kexec -f KERNEL ...      load and boot in one step");
}
