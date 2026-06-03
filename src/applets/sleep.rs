/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::thread;
use std::time::Duration;

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("sleep: missing operand");
        return 1;
    }

    let mut total_secs: f64 = 0.0;

    for arg in args {
        let (num_str, multiplier) = if arg.ends_with('d') {
            (&arg[..arg.len() - 1], 86400.0)
        } else if arg.ends_with('h') {
            (&arg[..arg.len() - 1], 3600.0)
        } else if arg.ends_with('m') {
            (&arg[..arg.len() - 1], 60.0)
        } else if arg.ends_with('s') {
            (&arg[..arg.len() - 1], 1.0)
        } else {
            (arg.as_str(), 1.0)
        };

        match num_str.parse::<f64>() {
            Ok(n) if n >= 0.0 => total_secs += n * multiplier,
            _ => {
                eprintln!("sleep: invalid time interval '{arg}'");
                return 1;
            }
        }
    }

    // Handle very large sleep values without overflow
    let duration = Duration::from_secs_f64(total_secs.min(u64::MAX as f64));
    thread::sleep(duration);
    0
}
