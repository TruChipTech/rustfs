/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use chrono::Local;

pub fn run(args: &[String]) -> i32 {
    let mut format = None;
    let mut utc = false;

    for arg in args {
        if arg.starts_with('+') {
            format = Some(arg[1..].to_string());
        } else if arg == "-u" || arg == "--utc" {
            utc = true;
        }
    }

    if utc {
        let now = chrono::Utc::now();
        if let Some(fmt) = format {
            println!("{}", now.format(&fmt));
        } else {
            println!("{}", now.format("%a %b %e %H:%M:%S UTC %Y"));
        }
    } else {
        let now = Local::now();
        if let Some(fmt) = format {
            println!("{}", now.format(&fmt));
        } else {
            println!("{}", now.format("%a %b %e %H:%M:%S %Z %Y"));
        }
    }

    0
}
