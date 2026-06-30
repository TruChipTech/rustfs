/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! pipe_progress — copy stdin to stdout, printing a dot to stderr every second.

use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

pub fn run(_args: &[String]) -> i32 {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let stderr = io::stderr();
    let mut r = stdin.lock();
    let mut w = stdout.lock();
    let mut e = stderr.lock();

    let mut buf = [0u8; 65536];
    let mut last = Instant::now();
    let tick = Duration::from_secs(1);
    loop {
        match r.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if w.write_all(&buf[..n]).is_err() {
                    break;
                }
                if last.elapsed() >= tick {
                    let _ = e.write_all(b".");
                    let _ = e.flush();
                    last = Instant::now();
                }
            }
            Err(ref err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
    let _ = w.flush();
    let _ = e.write_all(b"\n");
    0
}
