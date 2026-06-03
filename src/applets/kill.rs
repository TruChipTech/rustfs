/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! kill — send signals to processes

pub fn run(args: &[String]) -> i32 {
    let mut signal = libc::SIGTERM;
    let mut pids: Vec<i32> = Vec::new();
    let mut list_signals = false;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-l" || arg == "--list" {
            list_signals = true;
        } else if arg == "-s" {
            i += 1;
            if i < args.len() {
                signal = parse_signal(&args[i]);
            }
        } else if arg.starts_with('-') && arg.len() > 1 {
            let sig_str = &arg[1..];
            // Could be -9, -KILL, -SIGKILL
            let parsed = parse_signal(sig_str);
            if parsed != 0 {
                signal = parsed;
            } else if let Ok(pid) = sig_str.parse::<i32>() {
                // Negative PID (process group)
                pids.push(-pid);
            }
        } else if let Ok(pid) = arg.parse::<i32>() {
            pids.push(pid);
        } else {
            eprintln!("kill: invalid argument: {arg}");
            return 1;
        }
        i += 1;
    }

    if list_signals {
        print_signals();
        return 0;
    }

    if pids.is_empty() {
        eprintln!("Usage: kill [-s SIGNAL | -SIGNAL] PID...");
        return 1;
    }

    let mut exit_code = 0;
    for pid in &pids {
        if unsafe { libc::kill(*pid, signal) } != 0 {
            eprintln!("kill: ({pid}): {}", std::io::Error::last_os_error());
            exit_code = 1;
        }
    }
    exit_code
}

fn parse_signal(s: &str) -> i32 {
    // Try numeric
    if let Ok(n) = s.parse::<i32>() {
        return n;
    }

    let upper = s.to_uppercase();
    let name = upper.strip_prefix("SIG").unwrap_or(&upper);
    match name {
        "HUP" => 1,
        "INT" => 2,
        "QUIT" => 3,
        "ILL" => 4,
        "TRAP" => 5,
        "ABRT" | "IOT" => 6,
        "BUS" => 7,
        "FPE" => 8,
        "KILL" => 9,
        "USR1" => 10,
        "SEGV" => 11,
        "USR2" => 12,
        "PIPE" => 13,
        "ALRM" => 14,
        "TERM" => 15,
        "STKFLT" => 16,
        "CHLD" => 17,
        "CONT" => 18,
        "STOP" => 19,
        "TSTP" => 20,
        "TTIN" => 21,
        "TTOU" => 22,
        "URG" => 23,
        "XCPU" => 24,
        "XFSZ" => 25,
        "VTALRM" => 26,
        "PROF" => 27,
        "WINCH" => 28,
        "IO" | "POLL" => 29,
        "PWR" => 30,
        "SYS" => 31,
        _ => 0,
    }
}

fn print_signals() {
    let signals = [
        (1, "HUP"), (2, "INT"), (3, "QUIT"), (4, "ILL"),
        (5, "TRAP"), (6, "ABRT"), (7, "BUS"), (8, "FPE"),
        (9, "KILL"), (10, "USR1"), (11, "SEGV"), (12, "USR2"),
        (13, "PIPE"), (14, "ALRM"), (15, "TERM"), (16, "STKFLT"),
        (17, "CHLD"), (18, "CONT"), (19, "STOP"), (20, "TSTP"),
        (21, "TTIN"), (22, "TTOU"), (23, "URG"), (24, "XCPU"),
        (25, "XFSZ"), (26, "VTALRM"), (27, "PROF"), (28, "WINCH"),
        (29, "IO"), (30, "PWR"), (31, "SYS"),
    ];
    for (num, name) in &signals {
        print!("{num:2}) SIG{name:<8}");
        if num % 4 == 0 { println!(); }
    }
    println!();
}
