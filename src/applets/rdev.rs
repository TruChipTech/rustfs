/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! rdev — query the device of the mounted root filesystem

pub fn run(_args: &[String]) -> i32 {
    // Find the device mounted at "/".
    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            if f.len() >= 2 && f[1] == "/" {
                println!("{} /", f[0]);
                return 0;
            }
        }
    }
    eprintln!("rdev: cannot determine root device");
    1
}
