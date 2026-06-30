/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! who — show who is logged on (parses utmp)
use chrono::TimeZone;
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let am_i = args.iter().any(|a| a == "am" || a == "-m");

    let data = match fs::read("/var/run/utmp") {
        Ok(d) => d,
        Err(_) => match fs::read("/run/utmp") {
            Ok(d) => d,
            Err(e) => { eprintln!("who: cannot read utmp: {e}"); return 1; }
        },
    };

    let my_tty = current_tty();
    let entry_size = 384;
    let n = data.len() / entry_size;
    for i in 0..n {
        let off = i * entry_size;
        let e = &data[off..off + entry_size];
        let ut_type = i16::from_ne_bytes([e[0], e[1]]);
        if ut_type != 7 { continue; } // USER_PROCESS only
        let user = cstr(&e[8..40]);
        let line = cstr(&e[40..72]);
        let host = cstr(&e[76..332]);
        let tv_sec = i64::from_ne_bytes([
            e[340], e[341], e[342], e[343], e[344], e[345], e[346], e[347],
        ]);
        if user.is_empty() { continue; }
        if am_i {
            if let Some(t) = &my_tty {
                if &line != t { continue; }
            }
        }
        let when = chrono::Local
            .timestamp_opt(tv_sec, 0)
            .single()
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default();
        if host.is_empty() {
            println!("{user:<8} {line:<12} {when}");
        } else {
            println!("{user:<8} {line:<12} {when} ({host})");
        }
    }
    0
}

fn current_tty() -> Option<String> {
    let p = std::fs::read_link("/proc/self/fd/0").ok()?;
    let s = p.to_string_lossy();
    Some(s.strip_prefix("/dev/").unwrap_or(&s).to_string())
}

fn cstr(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}
