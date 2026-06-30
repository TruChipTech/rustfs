/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! pkill — signal processes by name (pattern)
use crate::applets::pgrep;

pub fn run(args: &[String]) -> i32 {
    let mut signal = libc::SIGTERM;
    let mut rest: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if let Some(sig) = a.strip_prefix("-SIG") {
            signal = parse_signal(sig);
        } else if a.starts_with('-') && a.len() > 1 && a[1..].chars().all(|c| c.is_ascii_digit()) {
            signal = a[1..].parse().unwrap_or(libc::SIGTERM);
        } else if a == "-s" {
            i += 1;
            if let Some(v) = args.get(i) { signal = parse_signal(v); }
        } else {
            rest.push(a.clone());
        }
        i += 1;
    }

    let (matches, _) = pgrep::collect(&rest, "pkill");
    if matches.is_empty() {
        return 1;
    }
    for (pid, _) in &matches {
        unsafe { libc::kill(*pid, signal); }
    }
    0
}

fn parse_signal(s: &str) -> i32 {
    if let Ok(n) = s.parse::<i32>() { return n; }
    let upper = s.to_uppercase();
    let name = upper.strip_prefix("SIG").unwrap_or(&upper);
    match name {
        "HUP" => 1, "INT" => 2, "QUIT" => 3, "KILL" => 9,
        "TERM" => 15, "STOP" => 19, "CONT" => 18, "USR1" => 10, "USR2" => 12,
        _ => libc::SIGTERM,
    }
}
