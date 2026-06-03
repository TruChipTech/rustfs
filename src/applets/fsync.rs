/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! fsync — synchronize a file's state with storage device

use std::fs;
use std::os::unix::io::AsRawFd;

pub fn run(args: &[String]) -> i32 {
    let mut datasync = false;
    let mut files: Vec<String> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-d" | "--datasync" => datasync = true,
            "-h" | "--help" => {
                eprintln!("Usage: fsync [-d] FILE...");
                return 0;
            }
            s if !s.starts_with('-') => files.push(s.to_string()),
            other => {
                eprintln!("fsync: unknown option: {other}");
                return 1;
            }
        }
    }

    if files.is_empty() {
        eprintln!("Usage: fsync [-d] FILE...");
        return 1;
    }

    let mut exit_code = 0;
    for file in &files {
        match fs::File::open(file) {
            Ok(f) => {
                let fd = f.as_raw_fd();
                let ret = if datasync {
                    unsafe { libc::fdatasync(fd) }
                } else {
                    unsafe { libc::fsync(fd) }
                };
                if ret != 0 {
                    eprintln!("fsync: {file}: {}", std::io::Error::last_os_error());
                    exit_code = 1;
                }
            }
            Err(e) => {
                eprintln!("fsync: {file}: {e}");
                exit_code = 1;
            }
        }
    }
    exit_code
}
