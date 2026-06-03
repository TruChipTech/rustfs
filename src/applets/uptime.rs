/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::time::SystemTime;

pub fn run(_args: &[String]) -> i32 {
    #[cfg(unix)]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/uptime") {
            if let Some(secs_str) = content.split_whitespace().next() {
                if let Ok(secs) = secs_str.parse::<f64>() {
                    let total_secs = secs as u64;
                    let days = total_secs / 86400;
                    let hours = (total_secs % 86400) / 3600;
                    let mins = (total_secs % 3600) / 60;

                    let now = chrono::Local::now();
                    print!(" {} up ", now.format("%H:%M:%S"));

                    if days > 0 {
                        print!("{days} day{}, ", if days != 1 { "s" } else { "" });
                    }
                    println!("{hours:02}:{mins:02}");
                    return 0;
                }
            }
        }
    }

    // Fallback: show how long the process has been running
    let now = SystemTime::now();
    let _ = now;
    println!("uptime information not available on this platform");
    0
}
