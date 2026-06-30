/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! beep — sound the console speaker (falls back to the terminal bell).

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::os::unix::io::AsRawFd;
use std::thread;
use std::time::Duration;

const KIOCSOUND: libc::c_ulong = 0x4B2F;
const CLOCK_TICK_RATE: u32 = 1193180;

pub fn run(args: &[String]) -> i32 {
    let mut freq: u32 = 4000;
    let mut length_ms: u64 = 30;
    let mut reps: u32 = 1;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-f" => {
                if let Some(v) = args.get(i + 1).and_then(|s| s.parse().ok()) {
                    freq = v;
                    i += 1;
                }
            }
            "-l" => {
                if let Some(v) = args.get(i + 1).and_then(|s| s.parse().ok()) {
                    length_ms = v;
                    i += 1;
                }
            }
            "-r" => {
                if let Some(v) = args.get(i + 1).and_then(|s| s.parse().ok()) {
                    reps = v;
                    i += 1;
                }
            }
            "--help" => {
                eprintln!("Usage: beep [-f FREQ] [-l LENGTH_MS] [-r REPEATS]");
                return 0;
            }
            _ => {}
        }
        i += 1;
    }

    let console = OpenOptions::new().write(true).open("/dev/console").ok();

    for n in 0..reps.max(1) {
        if n > 0 {
            thread::sleep(Duration::from_millis(length_ms));
        }
        let mut used_ioctl = false;
        if let Some(c) = &console {
            let arg = if freq == 0 { 0 } else { (CLOCK_TICK_RATE / freq) as libc::c_ulong };
            unsafe {
                if libc::ioctl(c.as_raw_fd(), KIOCSOUND as _, arg) == 0 {
                    used_ioctl = true;
                }
            }
        }
        if !used_ioctl {
            let _ = io::stderr().write_all(b"\x07");
            let _ = io::stderr().flush();
        }
        thread::sleep(Duration::from_millis(length_ms));
        if used_ioctl {
            if let Some(c) = &console {
                unsafe {
                    libc::ioctl(c.as_raw_fd(), KIOCSOUND as _, 0 as libc::c_ulong);
                }
            }
        }
    }
    0
}
