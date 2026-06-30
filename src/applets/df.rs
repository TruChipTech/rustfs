/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
pub fn run(args: &[String]) -> i32 {
    let mut human_readable = false;

    for arg in args {
        match arg.as_str() {
            "-h" | "--human-readable" => human_readable = true,
            _ => {}
        }
    }

    #[cfg(unix)]
    {
        print_df_unix(human_readable);
    }

    #[cfg(not(unix))]
    {
        print_df_portable(human_readable);
    }

    0
}

#[cfg(not(unix))]
fn print_df_portable(human_readable: bool) {
    println!(
        "{:<20} {:>12} {:>12} {:>12} {:>6} {}",
        "Filesystem", "Size", "Used", "Available", "Use%", "Mounted on"
    );

    // On Windows, enumerate drives
    #[cfg(windows)]
    {
        use std::path::Path;
        for letter in b'A'..=b'Z' {
            let drive = format!("{}:\\", letter as char);
            if Path::new(&drive).exists() {
                // Use GetDiskFreeSpaceEx via std
                if let Ok(meta) = std::fs::metadata(&drive) {
                    let _ = meta;
                }
                println!(
                    "{:<20} {:>12} {:>12} {:>12} {:>6} {}",
                    drive, "-", "-", "-", "-", drive
                );
            }
        }
    }
    let _ = human_readable;
}

#[cfg(unix)]
fn print_df_unix(human_readable: bool) {
    use std::fs;

    println!(
        "{:<20} {:>12} {:>12} {:>12} {:>6} Mounted on",
        "Filesystem", "Size", "Used", "Available", "Use%"
    );

    if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            let device = parts[0];
            let mount_point = parts[1];

            // Skip pseudo-filesystems
            if device == "none" || device == "proc" || device == "sysfs" || device == "devpts" {
                continue;
            }

            unsafe {
                let mut stat: libc::statvfs = std::mem::zeroed();
                let path = std::ffi::CString::new(mount_point).unwrap();
                if libc::statvfs(path.as_ptr(), &mut stat) == 0 {
                    let block_size = stat.f_frsize as u64;
                    let total = stat.f_blocks as u64 * block_size;
                    let free = stat.f_bavail as u64 * block_size;
                    let used = total - (stat.f_bfree as u64 * block_size);
                    let use_pct = if total > 0 {
                        (used as f64 / total as f64 * 100.0) as u64
                    } else {
                        0
                    };

                    if human_readable {
                        println!(
                            "{:<20} {:>12} {:>12} {:>12} {:>5}% {}",
                            device,
                            human_size(total),
                            human_size(used),
                            human_size(free),
                            use_pct,
                            mount_point
                        );
                    } else {
                        println!(
                            "{:<20} {:>12} {:>12} {:>12} {:>5}% {}",
                            device,
                            total / 1024,
                            used / 1024,
                            free / 1024,
                            use_pct,
                            mount_point
                        );
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
    let mut size = bytes as f64;
    for unit in UNITS {
        if size < 1024.0 {
            return format!("{:.1}{unit}", size);
        }
        size /= 1024.0;
    }
    format!("{:.1}P", size)
}
