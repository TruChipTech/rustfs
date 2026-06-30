/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub fn run(args: &[String]) -> i32 {
    let mut signal_name = "TERM";
    let mut preserve_status = false;
    let mut foreground = false;
    let mut duration_str: Option<&str> = None;
    let mut cmd: Vec<&str> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--signal" => {
                i += 1;
                if i < args.len() {
                    signal_name = &args[i];
                }
            }
            "--preserve-status" => preserve_status = true,
            "--foreground"      => foreground = true,
            "--"                => {
                i += 1;
                for a in &args[i..] { cmd.push(a); }
                break;
            }
            s if s.starts_with("--signal=") => signal_name = &s[9..],
            s if s.starts_with('-')         => {} // unknown flag, skip
            s => {
                if duration_str.is_none() {
                    duration_str = Some(s);
                } else {
                    for a in &args[i..] { cmd.push(a); }
                    break;
                }
            }
        }
        i += 1;
    }

    let _ = foreground; // not implemented at OS level; child inherits terminal

    let secs = match duration_str.and_then(parse_duration) {
        Some(s) => s,
        None => {
            eprintln!("timeout: missing timeout duration");
            eprintln!("Usage: timeout DURATION COMMAND [ARG]...");
            return 125;
        }
    };

    if cmd.is_empty() {
        eprintln!("timeout: missing command");
        return 125;
    }

    let mut child = match Command::new(cmd[0]).args(&cmd[1..]).spawn() {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("timeout: '{}': not found", cmd[0]);
            return 127;
        }
        Err(e) => {
            eprintln!("timeout: cannot run '{}': {e}", cmd[0]);
            return 126;
        }
    };

    let pid = child.id();
    let (tx, rx) = mpsc::channel::<std::io::Result<std::process::ExitStatus>>();

    thread::spawn(move || {
        let _ = tx.send(child.wait());
    });

    match rx.recv_timeout(Duration::from_secs_f64(secs)) {
        Ok(Ok(status)) => status.code().unwrap_or(1),
        Ok(Err(_)) => 1,
        Err(_) => {
            // Timed out — send the configured signal
            let signum = name_to_signum(signal_name);
            unsafe { libc::kill(pid as libc::pid_t, signum) };

            if preserve_status {
                // Wait up to 5 s for child to exit after signal
                match rx.recv_timeout(Duration::from_secs(5)) {
                    Ok(Ok(s)) => s.code().unwrap_or(1),
                    _ => 124,
                }
            } else {
                124
            }
        }
    }
}

fn parse_duration(s: &str) -> Option<f64> {
    if s.is_empty() { return None; }
    let (num, mult) = if let Some(p) = s.strip_suffix('s') {
        (p, 1.0f64)
    } else if let Some(p) = s.strip_suffix('m') {
        (p, 60.0)
    } else if let Some(p) = s.strip_suffix('h') {
        (p, 3600.0)
    } else if let Some(p) = s.strip_suffix('d') {
        (p, 86400.0)
    } else {
        (s, 1.0)
    };
    num.parse::<f64>().ok().map(|n| n * mult)
}

fn name_to_signum(name: &str) -> libc::c_int {
    let upper = name.to_uppercase();
    let trimmed = upper.strip_prefix("SIG").unwrap_or(upper.as_str());
    match trimmed {
        "TERM" | "15" => libc::SIGTERM,
        "KILL" | "9"  => libc::SIGKILL,
        "INT"  | "2"  => libc::SIGINT,
        "HUP"  | "1"  => libc::SIGHUP,
        "USR1" | "10" => libc::SIGUSR1,
        "USR2" | "12" => libc::SIGUSR2,
        "QUIT" | "3"  => libc::SIGQUIT,
        n => n.parse().unwrap_or(libc::SIGTERM),
    }
}
