/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! fbset — show and modify frame buffer device settings

use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut device = "/dev/fb0".to_string();
    let mut show_info = true;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-fb" => {
                i += 1;
                if i < args.len() { device = args[i].clone(); }
            }
            "-i" | "--info" => show_info = true,
            "-h" | "--help" => {
                eprintln!("Usage: fbset [-fb device] [-i]");
                return 0;
            }
            _ => {}
        }
        i += 1;
    }

    if show_info {
        return show_fb_info(&device);
    }

    0
}

fn show_fb_info(device: &str) -> i32 {
    // Try to get info via ioctl on the framebuffer device
    // Fallback: read from sysfs
    let fb_name = std::path::Path::new(device)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("fb0");

    let sysfs_path = format!("/sys/class/graphics/{fb_name}");

    if !std::path::Path::new(&sysfs_path).exists() {
        eprintln!("fbset: {device}: No such device");
        return 1;
    }

    println!("mode \"current\"");

    // Read geometry
    if let Ok(vinfo) = fs::read_to_string(format!("{sysfs_path}/virtual_size")) {
        let parts: Vec<&str> = vinfo.trim().split(',').collect();
        if parts.len() >= 2 {
            println!("    geometry {} {}", parts[0], parts[1]);
        }
    }

    if let Ok(bpp) = fs::read_to_string(format!("{sysfs_path}/bits_per_pixel")) {
        println!("    depth {}", bpp.trim());
    }

    if let Ok(stride) = fs::read_to_string(format!("{sysfs_path}/stride")) {
        println!("    stride {}", stride.trim());
    }

    if let Ok(name) = fs::read_to_string(format!("{sysfs_path}/name")) {
        println!("    name \"{}\"", name.trim());
    }

    println!("endmode");
    0
}
