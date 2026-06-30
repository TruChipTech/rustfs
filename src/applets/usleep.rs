/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! usleep — pause for N microseconds
use std::thread;
use std::time::Duration;

pub fn run(args: &[String]) -> i32 {
    let usec: u64 = args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    thread::sleep(Duration::from_micros(usec));
    0
}
