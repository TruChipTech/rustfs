/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! last — show listing of last logged in users

use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut count: Option<usize> = None;
    let mut wtmp_file = "/var/log/wtmp".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-n" => {
                i += 1;
                if i < args.len() { count = args[i].parse().ok(); }
            }
            "-f" => {
                i += 1;
                if i < args.len() { wtmp_file = args[i].clone(); }
            }
            "-h" | "--help" => {
                eprintln!("Usage: last [-n NUM] [-f FILE]");
                return 0;
            }
            _ => {}
        }
        i += 1;
    }

    // Read wtmp file
    let data = match fs::read(&wtmp_file) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("last: cannot open {wtmp_file}: {e}");
            return 1;
        }
    };

    // utmp entry size is 384 bytes on x86_64 Linux
    let entry_size = 384;
    if data.len() < entry_size {
        println!("\nwtmp begins (empty)");
        return 0;
    }

    let num_entries = data.len() / entry_size;
    let max_show = count.unwrap_or(num_entries);
    let mut shown = 0;

    // Parse entries from end (most recent first)
    for i in (0..num_entries).rev() {
        if shown >= max_show { break; }

        let offset = i * entry_size;
        let entry = &data[offset..offset + entry_size];

        // ut_type at offset 0 (i16)
        let ut_type = i16::from_ne_bytes([entry[0], entry[1]]);

        // Skip non-user entries
        // 7 = USER_PROCESS, 8 = DEAD_PROCESS
        if ut_type != 7 && ut_type != 8 { continue; }

        // ut_user at offset 8, 32 bytes
        let user = extract_string(&entry[8..40]);
        // ut_line at offset 40, 32 bytes  
        let line = extract_string(&entry[40..72]);
        // ut_host at offset 76, 256 bytes
        let host = extract_string(&entry[76..332]);
        // ut_tv at offset 332 (timeval: 8 bytes for tv_sec)
        let tv_sec = i64::from_ne_bytes([
            entry[340], entry[341], entry[342], entry[343],
            entry[344], entry[345], entry[346], entry[347],
        ]);

        if user.is_empty() && line.is_empty() { continue; }

        let time_str = format_time(tv_sec);
        let status = if ut_type == 8 { "gone" } else { "still logged in" };

        if host.is_empty() {
            println!("{user:<10} {line:<12} {time_str:<24} ({status})");
        } else {
            println!("{user:<10} {line:<12} {host:<16} {time_str:<24} ({status})");
        }
        shown += 1;
    }

    println!("\nwtmp begins {wtmp_file}");
    0
}

fn extract_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    std::str::from_utf8(&bytes[..end]).unwrap_or("").to_string()
}

fn format_time(timestamp: i64) -> String {
    if timestamp == 0 { return String::new(); }
    // Use libc localtime
    let t = timestamp as libc::time_t;
    let tm = unsafe { libc::localtime(&t) };
    if tm.is_null() { return format!("{timestamp}"); }
    let tm = unsafe { &*tm };

    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                  "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

    format!("{} {} {:2} {:02}:{:02}",
        days[tm.tm_wday as usize % 7],
        months[tm.tm_mon as usize % 12],
        tm.tm_mday, tm.tm_hour, tm.tm_min)
}
