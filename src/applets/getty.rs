/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! getty — open a terminal and set its mode

use std::ffi::CString;
use std::io::{self, Write};

pub fn run(args: &[String]) -> i32 {
    let mut baud = "9600".to_string();
    let mut tty_device = String::new();
    let mut term_type = "linux".to_string();
    let mut login_program = "/sbin/login".to_string();
    let mut noclear = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-L" | "--local-line" => {}
            "-n" | "--noclear" => noclear = true,
            "-l" => {
                i += 1;
                if i < args.len() { login_program = args[i].clone(); }
            }
            "-h" | "--help" => {
                eprintln!("Usage: getty [-n] [-l LOGIN] BAUD TTY [TERM]");
                return 0;
            }
            s if !s.starts_with('-') => {
                if baud == "9600" && tty_device.is_empty() {
                    baud = s.to_string();
                } else if tty_device.is_empty() {
                    tty_device = s.to_string();
                } else {
                    term_type = s.to_string();
                }
            }
            _ => {}
        }
        i += 1;
    }

    if tty_device.is_empty() {
        eprintln!("Usage: getty [-n] [-l LOGIN] BAUD TTY [TERM]");
        return 1;
    }

    // Open the TTY
    let tty_path = if tty_device.starts_with('/') {
        tty_device.clone()
    } else {
        format!("/dev/{tty_device}")
    };

    let c_path = match CString::new(tty_path.as_str()) {
        Ok(p) => p,
        Err(_) => {
            eprintln!("getty: invalid tty path");
            return 1;
        }
    };

    // Open TTY for read/write
    let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDWR | libc::O_NOCTTY) };
    if fd < 0 {
        eprintln!("getty: cannot open {tty_path}: {}", io::Error::last_os_error());
        return 1;
    }

    // Make this the controlling terminal
    unsafe {
        libc::setsid();
        libc::ioctl(fd, libc::TIOCSCTTY as _, 0);
        libc::dup2(fd, 0);
        libc::dup2(fd, 1);
        libc::dup2(fd, 2);
        if fd > 2 { libc::close(fd); }
    }

    // Set TERM environment variable
    std::env::set_var("TERM", &term_type);

    // Clear screen unless -n
    if !noclear {
        let _ = io::stdout().write_all(b"\x1B[2J\x1B[H");
    }

    // Print hostname
    let mut hostname_buf = [0u8; 256];
    let hostname = unsafe {
        if libc::gethostname(hostname_buf.as_mut_ptr() as *mut libc::c_char, 256) == 0 {
            let len = hostname_buf.iter().position(|&b| b == 0).unwrap_or(0);
            std::str::from_utf8(&hostname_buf[..len]).unwrap_or("localhost")
        } else {
            "localhost"
        }
    };

    print!("{hostname} login: ");
    let _ = io::stdout().flush();

    // Read username
    let mut username = String::new();
    if io::stdin().read_line(&mut username).is_err() {
        return 1;
    }
    let username = username.trim();

    // Exec login program
    let c_login = CString::new(login_program.as_str()).unwrap();
    let c_arg0 = CString::new("login").unwrap();
    let c_user = CString::new(username).unwrap();
    let args_arr = [c_arg0.as_ptr(), c_user.as_ptr(), std::ptr::null()];

    unsafe { libc::execv(c_login.as_ptr(), args_arr.as_ptr()) };
    eprintln!("getty: exec {login_program} failed: {}", io::Error::last_os_error());
    1
}
