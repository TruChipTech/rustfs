/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! klogd — kernel log daemon

pub fn run(args: &[String]) -> i32 {
    let mut foreground = false;
    let mut console_level: i32 = -1;

    for arg in args {
        match arg.as_str() {
            "-n" => foreground = true,
            "-c" => console_level = 7,
            "-h" | "--help" => {
                eprintln!("Usage: klogd [-n] [-c LEVEL]");
                return 0;
            }
            s if s.starts_with("-c") => {
                console_level = s[2..].parse().unwrap_or(7);
            }
            _ => {}
        }
    }

    if !foreground {
        let pid = unsafe { libc::fork() };
        if pid < 0 {
            eprintln!("klogd: fork failed");
            return 1;
        }
        if pid > 0 { return 0; }
        unsafe { libc::setsid(); }
    }

    // Set console log level
    if console_level >= 0 {
        unsafe { libc::klogctl(8, std::ptr::null_mut(), console_level) };
    }

    // Open syslog
    let ident = std::ffi::CString::new("kernel").unwrap();
    unsafe { libc::openlog(ident.as_ptr(), libc::LOG_NDELAY | libc::LOG_PID, libc::LOG_KERN) };

    eprintln!("klogd: started");

    // Read kernel messages
    let buf_size = 8192;
    let mut buf = vec![0u8; buf_size];

    loop {
        // klogctl(2, buf, len) — SYSLOG_ACTION_READ (blocking)
        let ret = unsafe {
            libc::klogctl(2, buf.as_mut_ptr() as *mut libc::c_char, buf_size as libc::c_int)
        };

        if ret < 0 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::EINTR) { continue; }
            eprintln!("klogd: klogctl error: {err}");
            break;
        }

        if ret == 0 { continue; }

        let msg = &buf[..ret as usize];
        if let Ok(text) = std::str::from_utf8(msg) {
            for line in text.lines() {
                // Parse priority from <N> prefix
                let (priority, message) = if line.starts_with('<') {
                    if let Some(end) = line.find('>') {
                        let prio: i32 = line[1..end].parse().unwrap_or(6);
                        (prio, &line[end + 1..])
                    } else {
                        (6, line)
                    }
                } else {
                    (6, line)
                };

                let c_msg = std::ffi::CString::new(message).unwrap_or_default();
                unsafe { libc::syslog(priority, c_msg.as_ptr()) };
            }
        }
    }

    unsafe { libc::closelog() };
    0
}
