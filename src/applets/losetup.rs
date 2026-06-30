/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! losetup — set up and control loop devices

use std::fs;
use std::io;

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        return show_all_loops();
    }

    let mut detach = false;
    let mut detach_all = false;
    let mut find_free = false;
    let mut read_only = false;
    let mut offset: u64 = 0;
    let mut device: Option<String> = None;
    let mut file: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--detach" => detach = true,
            "-D" | "--detach-all" => detach_all = true,
            "-f" | "--find" => find_free = true,
            "-r" | "--read-only" => read_only = true,
            "-o" | "--offset" => {
                i += 1;
                if i < args.len() { offset = args[i].parse().unwrap_or(0); }
            }
            "-h" | "--help" => {
                eprintln!("Usage: losetup [-d DEVICE] [-D] [-f] [-r] [-o OFFSET] [DEVICE] [FILE]");
                return 0;
            }
            s if !s.starts_with('-') => {
                if device.is_none() {
                    device = Some(s.to_string());
                } else {
                    file = Some(s.to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }

    if detach_all {
        return detach_all_loops();
    }

    if detach {
        let dev = device.unwrap_or_default();
        if dev.is_empty() {
            eprintln!("losetup: -d requires a device");
            return 1;
        }
        return detach_loop(&dev);
    }

    if find_free {
        match find_free_loop() {
            Some(dev) => {
                if let Some(f) = file.or(device) {
                    return setup_loop(&dev, &f, read_only, offset);
                }
                println!("{dev}");
                return 0;
            }
            None => {
                eprintln!("losetup: no free loop device found");
                return 1;
            }
        }
    }

    if let (Some(dev), Some(f)) = (device, file) {
        return setup_loop(&dev, &f, read_only, offset);
    }

    show_all_loops()
}

fn show_all_loops() -> i32 {
    // Read /sys/block/loop*/loop/backing_file
    let entries = match fs::read_dir("/sys/block") {
        Ok(e) => e,
        Err(_) => { return 0; }
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("loop") { continue; }

        let backing = format!("/sys/block/{name}/loop/backing_file");
        if let Ok(file) = fs::read_to_string(&backing) {
            let file = file.trim();
            let offset_path = format!("/sys/block/{name}/loop/offset");
            let offset = fs::read_to_string(&offset_path).unwrap_or_default();
            println!("/dev/{name}: []: ({file}), offset {}", offset.trim());
        }
    }
    0
}

fn find_free_loop() -> Option<String> {
    // Use LOOP_CTL_GET_FREE ioctl
    let path = c"/dev/loop-control";
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDWR) };
    if fd < 0 { return None; }

    const LOOP_CTL_GET_FREE: libc::c_ulong = 0x4C82;
    let num = unsafe { libc::ioctl(fd, LOOP_CTL_GET_FREE as libc::Ioctl) };
    unsafe { libc::close(fd) };

    if num < 0 { None } else { Some(format!("/dev/loop{num}")) }
}

fn setup_loop(device: &str, file: &str, read_only: bool, _offset: u64) -> i32 {
    let flags = if read_only { libc::O_RDONLY } else { libc::O_RDWR };

    let c_file = std::ffi::CString::new(file).unwrap();
    let file_fd = unsafe { libc::open(c_file.as_ptr(), flags) };
    if file_fd < 0 {
        eprintln!("losetup: cannot open {file}: {}", io::Error::last_os_error());
        return 1;
    }

    let c_dev = std::ffi::CString::new(device).unwrap();
    let dev_fd = unsafe { libc::open(c_dev.as_ptr(), flags) };
    if dev_fd < 0 {
        eprintln!("losetup: cannot open {device}: {}", io::Error::last_os_error());
        unsafe { libc::close(file_fd) };
        return 1;
    }

    const LOOP_SET_FD: libc::c_ulong = 0x4C00;
    let ret = unsafe { libc::ioctl(dev_fd, LOOP_SET_FD as libc::Ioctl, file_fd) };
    unsafe {
        libc::close(file_fd);
        libc::close(dev_fd);
    }

    if ret < 0 {
        eprintln!("losetup: LOOP_SET_FD failed: {}", io::Error::last_os_error());
        return 1;
    }
    0
}

fn detach_loop(device: &str) -> i32 {
    let c_dev = std::ffi::CString::new(device).unwrap();
    let fd = unsafe { libc::open(c_dev.as_ptr(), libc::O_RDONLY) };
    if fd < 0 {
        eprintln!("losetup: cannot open {device}: {}", io::Error::last_os_error());
        return 1;
    }

    const LOOP_CLR_FD: libc::c_ulong = 0x4C01;
    let ret = unsafe { libc::ioctl(fd, LOOP_CLR_FD as libc::Ioctl, 0) };
    unsafe { libc::close(fd) };

    if ret < 0 {
        eprintln!("losetup: LOOP_CLR_FD failed: {}", io::Error::last_os_error());
        return 1;
    }
    0
}

fn detach_all_loops() -> i32 {
    let entries = match fs::read_dir("/sys/block") {
        Ok(e) => e,
        Err(_) => return 0,
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("loop") { continue; }
        let backing = format!("/sys/block/{name}/loop/backing_file");
        if fs::metadata(&backing).is_ok() {
            let dev = format!("/dev/{name}");
            detach_loop(&dev);
        }
    }
    0
}
