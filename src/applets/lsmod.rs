/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! lsmod — show the status of modules in the Linux Kernel

use std::fs;

pub fn run(_args: &[String]) -> i32 {
    let content = match fs::read_to_string("/proc/modules") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("lsmod: {e}");
            return 1;
        }
    };

    println!("{:<28} {:>8}  Used by", "Module", "Size");

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let name = parts[0];
            let size = parts[1];
            let used_count = parts[2];
            let used_by = if parts.len() >= 4 {
                parts[3].trim_end_matches(',').replace(',', ", ")
            } else {
                String::new()
            };

            if used_by.is_empty() || used_by == "-" {
                println!("{name:<28} {size:>8}  {used_count}");
            } else {
                println!("{name:<28} {size:>8}  {used_count} {used_by}");
            }
        }
    }

    0
}
