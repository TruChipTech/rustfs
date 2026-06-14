/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let all = args.iter().any(|a| a == "--all");
    let ignore: usize = args.windows(2)
        .find(|w| w[0] == "--ignore")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or_else(|| {
            args.iter()
                .filter_map(|a| a.strip_prefix("--ignore="))
                .filter_map(|s| s.parse().ok())
                .next()
                .unwrap_or(0)
        });

    let n = if all { count_total() } else { count_online() };
    println!("{}", n.saturating_sub(ignore).max(1));
    0
}

fn count_total() -> usize {
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        let n = content.lines().filter(|l| l.starts_with("processor")).count();
        if n > 0 { return n; }
    }
    count_online()
}

fn count_online() -> usize {
    let n = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
    if n > 0 { n as usize } else { 1 }
}
