/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! rtcwake — enter a system sleep state until a specified wakeup time
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut seconds: Option<u64> = None;
    let mut at_time: Option<i64> = None;
    let mut mode = "standby".to_string();
    let mut rtc = "rtc0".to_string();
    let mut dry = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--seconds" => { i += 1; seconds = args.get(i).and_then(|s| s.parse().ok()); }
            "-t" | "--time" => { i += 1; at_time = args.get(i).and_then(|s| s.parse().ok()); }
            "-m" | "--mode" => { i += 1; if let Some(m) = args.get(i) { mode = m.clone(); } }
            "-d" | "--device" => { i += 1; if let Some(d) = args.get(i) { rtc = d.clone(); } }
            "-n" | "--dry-run" => dry = true,
            _ => {}
        }
        i += 1;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let alarm = match (seconds, at_time) {
        (Some(s), _) => now + s as i64,
        (_, Some(t)) => t,
        _ => { eprintln!("rtcwake: must specify -s SECONDS or -t TIME"); return 1; }
    };

    let alarm_path = format!("/sys/class/rtc/{rtc}/wakealarm");
    // Clear any existing alarm first.
    let _ = fs::write(&alarm_path, "0\n");
    if let Err(e) = fs::write(&alarm_path, format!("{alarm}\n")) {
        eprintln!("rtcwake: {alarm_path}: {e}");
        return 1;
    }

    if dry || mode == "no" {
        println!("rtcwake: alarm set for {alarm}; not entering sleep");
        return 0;
    }

    if mode == "on" || mode == "disable" {
        return 0;
    }

    if let Err(e) = fs::write("/sys/power/state", format!("{mode}\n")) {
        eprintln!("rtcwake: /sys/power/state: {e}");
        return 1;
    }
    0
}
