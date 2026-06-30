/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! rdate — get/set the system time from an RFC 868 (time protocol) server.

use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;

// Seconds between 1900-01-01 (RFC 868 epoch) and 1970-01-01 (Unix epoch).
const RFC868_OFFSET: u32 = 2_208_988_800;

pub fn run(args: &[String]) -> i32 {
    let mut set = false;
    let mut print = false;
    let mut host: Option<String> = None;

    for a in args {
        match a.as_str() {
            "-s" => set = true,
            "-p" => print = true,
            "--help" => {
                eprintln!("Usage: rdate [-s] [-p] HOST");
                return 0;
            }
            s if s.starts_with('-') && s.len() > 1 => {
                eprintln!("rdate: unknown option '{s}'");
                return 1;
            }
            _ => host = Some(a.clone()),
        }
    }
    // Default action (no flags) is to set the clock.
    if !set && !print {
        set = true;
    }

    let host = match host {
        Some(h) => h,
        None => {
            eprintln!("rdate: missing HOST");
            return 1;
        }
    };

    let addr = format!("{host}:37");
    let mut stream = match TcpStream::connect(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("rdate: cannot connect to {host}: {e}");
            return 1;
        }
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));

    let mut buf = [0u8; 4];
    if let Err(e) = stream.read_exact(&mut buf) {
        eprintln!("rdate: read failed: {e}");
        return 1;
    }
    let secs1900 = u32::from_be_bytes(buf);
    if secs1900 < RFC868_OFFSET {
        eprintln!("rdate: invalid time received");
        return 1;
    }
    let unix = (secs1900 - RFC868_OFFSET) as i64;

    if print {
        println!("{unix}");
    }
    if set {
        let tv = libc::timeval {
            tv_sec: unix as libc::time_t,
            tv_usec: 0,
        };
        let rc = unsafe { libc::settimeofday(&tv, std::ptr::null()) };
        if rc != 0 {
            eprintln!("rdate: cannot set time: {}", std::io::Error::last_os_error());
            return 1;
        }
    }
    0
}
