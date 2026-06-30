/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! time — run a program and report how long it took
use std::process::Command;
use std::time::Instant;

pub fn run(args: &[String]) -> i32 {
    let mut idx = 0;
    while idx < args.len() && args[idx].starts_with('-') && args[idx] != "-" {
        // Accept and ignore -p / -v style flags.
        idx += 1;
    }
    if idx >= args.len() {
        eprintln!("Usage: time COMMAND [args]");
        return 1;
    }

    let start = Instant::now();
    let (mut ru_before, mut ru_after): (libc::rusage, libc::rusage) =
        unsafe { (std::mem::zeroed(), std::mem::zeroed()) };
    unsafe { libc::getrusage(libc::RUSAGE_CHILDREN, &mut ru_before); }

    let status = Command::new(&args[idx]).args(&args[idx + 1..]).status();
    let elapsed = start.elapsed();
    unsafe { libc::getrusage(libc::RUSAGE_CHILDREN, &mut ru_after); }

    let code = match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => { eprintln!("time: {}: {e}", args[idx]); return 127; }
    };

    let user = tv_diff(ru_after.ru_utime, ru_before.ru_utime);
    let sys = tv_diff(ru_after.ru_stime, ru_before.ru_stime);
    eprintln!("real\t{}", fmt(elapsed.as_secs_f64()));
    eprintln!("user\t{}", fmt(user));
    eprintln!("sys\t{}", fmt(sys));
    code
}

fn tv_diff(a: libc::timeval, b: libc::timeval) -> f64 {
    (a.tv_sec - b.tv_sec) as f64 + (a.tv_usec - b.tv_usec) as f64 / 1_000_000.0
}

fn fmt(secs: f64) -> String {
    let m = (secs / 60.0) as u64;
    let s = secs - (m as f64) * 60.0;
    format!("{m}m{s:.3}s")
}
