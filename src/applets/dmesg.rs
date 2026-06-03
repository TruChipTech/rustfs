/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! dmesg — print or control the kernel ring buffer

use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut clear = false;
    let mut level: Option<&str> = None;
    let mut human_readable = false;
    let mut follow = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--clear" => clear = true,
            "-C" | "--clear-only" => {
                // Clear without printing
                return clear_ring_buffer();
            }
            "-H" | "--human" => human_readable = true,
            "-w" | "--follow" => follow = true,
            "-l" | "--level" => {
                i += 1;
                if i < args.len() { level = Some(&args[i]); }
            }
            "-h" | "--help" => {
                eprintln!("Usage: dmesg [-c] [-C] [-H] [-l LEVEL] [-w]");
                return 0;
            }
            _ => {}
        }
        i += 1;
    }

    // Read from /dev/kmsg or /proc/kmsg
    let content = if std::path::Path::new("/dev/kmsg").exists() {
        // /dev/kmsg requires special handling; fall back to syslog
        read_kernel_log()
    } else {
        read_kernel_log()
    };

    let content = match content {
        Ok(c) => c,
        Err(e) => {
            eprintln!("dmesg: {e}");
            return 1;
        }
    };

    for line in content.lines() {
        if let Some(lvl) = level {
            // Filter by level
            if let Some(log_level) = extract_level(line) {
                if !matches_level(log_level, lvl) {
                    continue;
                }
            }
        }

        if human_readable {
            println!("{}", format_human(line));
        } else {
            println!("{line}");
        }
    }

    if follow {
        eprintln!("dmesg: --follow requires reading from /dev/kmsg (not implemented)");
    }

    if clear {
        let _ = clear_ring_buffer();
    }

    0
}

fn read_kernel_log() -> Result<String, String> {
    // Try /var/log/dmesg first, then use klogctl syscall
    if let Ok(content) = fs::read_to_string("/var/log/dmesg") {
        return Ok(content);
    }

    // Use klogctl(3, buf, len) — SYSLOG_ACTION_READ_ALL
    let buf_size: usize = 1024 * 512; // 512 KB
    let mut buf = vec![0u8; buf_size];
    let ret = unsafe {
        libc::klogctl(3, buf.as_mut_ptr() as *mut libc::c_char, buf_size as libc::c_int)
    };
    if ret < 0 {
        return Err(format!("cannot read kernel log: {}", std::io::Error::last_os_error()));
    }
    let len = ret as usize;
    buf.truncate(len);
    String::from_utf8(buf).map_err(|e| format!("invalid UTF-8 in kernel log: {e}"))
}

fn clear_ring_buffer() -> i32 {
    // klogctl(5, NULL, 0) — SYSLOG_ACTION_CLEAR
    let ret = unsafe { libc::klogctl(5, std::ptr::null_mut(), 0) };
    if ret < 0 {
        eprintln!("dmesg: clear failed: {}", std::io::Error::last_os_error());
        return 1;
    }
    0
}

fn extract_level(line: &str) -> Option<u8> {
    // Kernel log format: <LEVEL>message or [timestamp] message
    if line.starts_with('<') {
        if let Some(end) = line.find('>') {
            if let Ok(lvl) = line[1..end].parse::<u8>() {
                return Some(lvl);
            }
        }
    }
    None
}

fn matches_level(log_level: u8, filter: &str) -> bool {
    let target = match filter {
        "emerg" => 0,
        "alert" => 1,
        "crit" => 2,
        "err" => 3,
        "warn" | "warning" => 4,
        "notice" => 5,
        "info" => 6,
        "debug" => 7,
        _ => return true,
    };
    log_level <= target
}

fn format_human(line: &str) -> String {
    // Strip priority prefix if present
    if line.starts_with('<') {
        if let Some(end) = line.find('>') {
            return line[end + 1..].to_string();
        }
    }
    line.to_string()
}
