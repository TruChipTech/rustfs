/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! watch — execute a program periodically, showing output fullscreen
use std::process::Command;
use std::thread;
use std::time::Duration;

pub fn run(args: &[String]) -> i32 {
    let mut interval = 2.0f64;
    let mut no_title = false;
    let mut idx = 0;

    while idx < args.len() {
        let a = &args[idx];
        match a.as_str() {
            "-n" | "--interval" => { idx += 1; interval = args.get(idx).and_then(|s| s.parse().ok()).unwrap_or(2.0); }
            "-t" | "--no-title" => no_title = true,
            s if s.starts_with('-') => {}
            _ => break,
        }
        idx += 1;
    }

    if idx >= args.len() {
        eprintln!("Usage: watch [-n SEC] [-t] COMMAND");
        return 1;
    }
    let cmd_str = args[idx..].join(" ");

    loop {
        // Clear screen + home cursor.
        print!("\x1b[2J\x1b[H");
        if !no_title {
            println!("Every {interval}s: {cmd_str}\n");
        }
        let status = Command::new("sh").arg("-c").arg(&cmd_str).status();
        if status.is_err() {
            eprintln!("watch: failed to run command");
            return 1;
        }
        thread::sleep(Duration::from_secs_f64(interval));
    }
}
