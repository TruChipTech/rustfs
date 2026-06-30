/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! mdev — lightweight device manager for RustFS
//!
//! A simple device manager that handles hotplug events by scanning /sys or
//! processing uevent environment variables. Creates/removes device nodes
//! in /dev based on rules in /etc/mdev.conf.
//!
//! Usage:
//!   mdev -s              Scan /sys and populate /dev
//!   mdev                 Handle a single hotplug event (called by kernel)
//!   mdev -d              Run as daemon, listening for uevents

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::Command;

const MDEV_CONF: &str = "/etc/mdev.conf";

/// mdev.conf rule
#[derive(Debug, Clone)]
struct MdevRule {
    pattern: String,
    uid: u32,
    gid: u32,
    mode: u32,
    /// Optional: '=' create, '!' don't create, '>' move, '@' run script on add,
    /// '$' run script on remove, '*' run on both
    action: Option<char>,
    /// Symlink name or script path
    parameter: String,
}

pub fn run(args: &[String]) -> i32 {
    if args.iter().any(|a| a == "-s") {
        return scan_sys();
    }

    if args.iter().any(|a| a == "-d") {
        return run_daemon();
    }

    // Default: handle a single hotplug event from environment
    handle_hotplug_event()
}

/// Scan /sys/class and /sys/block to populate /dev
fn scan_sys() -> i32 {
    eprintln!("mdev: scanning /sys for devices...");

    let rules = load_rules();

    // Scan /sys/class/*
    scan_class_dir("/sys/class", &rules);

    // Scan /sys/block/*
    scan_block_dir("/sys/block", &rules);

    eprintln!("mdev: scan complete");
    0
}

fn scan_class_dir(base: &str, rules: &[MdevRule]) {
    let base_path = Path::new(base);
    if !base_path.is_dir() {
        return;
    }

    if let Ok(classes) = fs::read_dir(base_path) {
        for class in classes.flatten() {
            let class_path = class.path();
            if !class_path.is_dir() {
                continue;
            }

            if let Ok(devices) = fs::read_dir(&class_path) {
                for device in devices.flatten() {
                    let dev_path = device.path().join("dev");
                    if dev_path.exists() {
                        if let Some(dev_name) = device.file_name().to_str() {
                            if let Ok(dev_num) = fs::read_to_string(&dev_path) {
                                create_dev_node(dev_name, dev_num.trim(), false, rules);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn scan_block_dir(base: &str, rules: &[MdevRule]) {
    let base_path = Path::new(base);
    if !base_path.is_dir() {
        return;
    }

    if let Ok(devices) = fs::read_dir(base_path) {
        for device in devices.flatten() {
            let dev_path = device.path().join("dev");
            if dev_path.exists() {
                if let Some(dev_name) = device.file_name().to_str() {
                    if let Ok(dev_num) = fs::read_to_string(&dev_path) {
                        create_dev_node(dev_name, dev_num.trim(), true, rules);
                    }
                }
            }

            // Check for partitions
            if let Ok(parts) = fs::read_dir(device.path()) {
                for part in parts.flatten() {
                    let part_dev = part.path().join("dev");
                    if part_dev.exists() {
                        if let Some(part_name) = part.file_name().to_str() {
                            if let Ok(dev_num) = fs::read_to_string(&part_dev) {
                                create_dev_node(part_name, dev_num.trim(), true, rules);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Create a device node in /dev
fn create_dev_node(name: &str, dev_num: &str, is_block: bool, rules: &[MdevRule]) {
    let parts: Vec<&str> = dev_num.split(':').collect();
    if parts.len() != 2 {
        return;
    }
    let major: u32 = match parts[0].parse() {
        Ok(v) => v,
        Err(_) => return,
    };
    let minor: u32 = match parts[1].parse() {
        Ok(v) => v,
        Err(_) => return,
    };

    let dev_path = format!("/dev/{name}");

    // Check if already exists
    if Path::new(&dev_path).exists() {
        return;
    }

    // Find matching rule
    let rule = find_matching_rule(name, rules);

    let mode = rule.as_ref().map_or(0o660, |r| r.mode);
    let uid = rule.as_ref().map_or(0, |r| r.uid);
    let gid = rule.as_ref().map_or(0, |r| r.gid);

    // Check if rule says don't create
    if let Some(ref r) = rule {
        if r.action == Some('!') {
            return;
        }
    }

    let dev = libc::makedev(major, minor);
    let node_type = if is_block { libc::S_IFBLK } else { libc::S_IFCHR };

    let c_path = match std::ffi::CString::new(dev_path.as_str()) {
        Ok(p) => p,
        Err(_) => return,
    };

    let ret = unsafe { libc::mknod(c_path.as_ptr(), node_type | mode, dev) };
    if ret == 0 {
        unsafe { libc::chown(c_path.as_ptr(), uid, gid) };

        // Handle symlinks from rules
        if let Some(ref r) = rule {
            if !r.parameter.is_empty() {
                match r.action {
                    Some('=') | Some('>') => {
                        // Move to specified path or create symlink
                        let link_path = format!("/dev/{}", r.parameter);
                        let _ = symlink(&dev_path, &link_path);
                    }
                    Some('@') | Some('*') => {
                        // Run script on add
                        let _ = Command::new("/bin/sh")
                            .args(["-c", &r.parameter])
                            .env("MDEV", name)
                            .env("ACTION", "add")
                            .status();
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Handle a hotplug event from kernel (environment-based)
fn handle_hotplug_event() -> i32 {
    let action = std::env::var("ACTION").unwrap_or_default();
    let devname = std::env::var("DEVNAME").unwrap_or_default();
    let devpath = std::env::var("DEVPATH").unwrap_or_default();
    let major = std::env::var("MAJOR").unwrap_or_default();
    let minor = std::env::var("MINOR").unwrap_or_default();
    let subsystem = std::env::var("SUBSYSTEM").unwrap_or_default();

    if devname.is_empty() && devpath.is_empty() {
        eprintln!("mdev: no DEVNAME or DEVPATH set");
        return 1;
    }

    let name = if !devname.is_empty() {
        devname.clone()
    } else {
        devpath.rsplit('/').next().unwrap_or("").to_string()
    };

    if name.is_empty() {
        return 1;
    }

    let rules = load_rules();

    match action.as_str() {
        "add"
            if !major.is_empty() && !minor.is_empty() => {
                let dev_num = format!("{major}:{minor}");
                let is_block = subsystem == "block";
                create_dev_node(&name, &dev_num, is_block, &rules);
            }
        "remove" => {
            let dev_path = format!("/dev/{name}");
            if Path::new(&dev_path).exists() {
                let _ = fs::remove_file(&dev_path);
            }
            // Run remove scripts
            if let Some(rule) = find_matching_rule(&name, &rules) {
                if matches!(rule.action, Some('$') | Some('*')) && !rule.parameter.is_empty() {
                    let _ = Command::new("/bin/sh")
                        .args(["-c", &rule.parameter])
                        .env("MDEV", &name)
                        .env("ACTION", "remove")
                        .status();
                }
            }
        }
        _ => {}
    }

    0
}

/// Run as daemon, listening for kernel uevents
fn run_daemon() -> i32 {
    eprintln!("mdev: starting daemon mode");

    // First do initial scan
    scan_sys();

    // Set up netlink socket to receive uevents
    unsafe {
        let sock = libc::socket(libc::AF_NETLINK, libc::SOCK_DGRAM, 15 /* NETLINK_KOBJECT_UEVENT */);
        if sock < 0 {
            eprintln!("mdev: failed to create netlink socket");
            return 1;
        }

        let mut addr: libc::sockaddr_nl = std::mem::zeroed();
        addr.nl_family = libc::AF_NETLINK as u16;
        addr.nl_groups = 1; // UEVENT multicast group
        addr.nl_pid = libc::getpid() as u32;

        let ret = libc::bind(
            sock,
            &addr as *const libc::sockaddr_nl as *const libc::sockaddr,
            std::mem::size_of::<libc::sockaddr_nl>() as u32,
        );
        if ret < 0 {
            eprintln!("mdev: failed to bind netlink socket");
            libc::close(sock);
            return 1;
        }

        let mut buf = vec![0u8; 8192];
        loop {
            let len = libc::recv(sock, buf.as_mut_ptr() as *mut libc::c_void, buf.len(), 0);
            if len <= 0 {
                continue;
            }

            // Parse uevent message
            let msg = String::from_utf8_lossy(&buf[..len as usize]);
            let mut action = String::new();
            let mut devname = String::new();
            let mut major = String::new();
            let mut minor = String::new();
            let mut subsystem = String::new();

            for line in msg.split('\0') {
                if let Some(val) = line.strip_prefix("ACTION=") {
                    action = val.to_string();
                } else if let Some(val) = line.strip_prefix("DEVNAME=") {
                    devname = val.to_string();
                } else if let Some(val) = line.strip_prefix("MAJOR=") {
                    major = val.to_string();
                } else if let Some(val) = line.strip_prefix("MINOR=") {
                    minor = val.to_string();
                } else if let Some(val) = line.strip_prefix("SUBSYSTEM=") {
                    subsystem = val.to_string();
                }
            }

            if !devname.is_empty() {
                std::env::set_var("ACTION", &action);
                std::env::set_var("DEVNAME", &devname);
                std::env::set_var("MAJOR", &major);
                std::env::set_var("MINOR", &minor);
                std::env::set_var("SUBSYSTEM", &subsystem);
                handle_hotplug_event();
            }
        }
    }
}

/// Load rules from /etc/mdev.conf
fn load_rules() -> Vec<MdevRule> {
    let mut rules = Vec::new();

    let content = match fs::read_to_string(MDEV_CONF) {
        Ok(c) => c,
        Err(_) => return rules,
    };

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Format: <pattern> <uid>:<gid> <mode> [<action> <parameter>]
        // Example: tty[0-9]* 0:5 0660
        // Example: sd[a-z]* 0:6 0660 * /etc/mdev/storage.sh
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let pattern = parts[0].to_string();

        let (uid, gid) = if let Some((u, g)) = parts[1].split_once(':') {
            (
                u.parse::<u32>().unwrap_or(0),
                g.parse::<u32>().unwrap_or(0),
            )
        } else {
            (0, 0)
        };

        let mode = u32::from_str_radix(parts[2], 8).unwrap_or(0o660);

        let (action, parameter) = if parts.len() >= 5 {
            (parts[3].chars().next(), parts[4].to_string())
        } else if parts.len() == 4 {
            (parts[3].chars().next(), String::new())
        } else {
            (None, String::new())
        };

        rules.push(MdevRule {
            pattern,
            uid,
            gid,
            mode,
            action,
            parameter,
        });
    }

    rules
}

/// Find matching rule for a device name using glob-style pattern matching
fn find_matching_rule(name: &str, rules: &[MdevRule]) -> Option<MdevRule> {
    for rule in rules {
        if glob_match(&rule.pattern, name) {
            return Some(rule.clone());
        }
    }
    None
}

/// Simple glob pattern matching (supports * and [])
fn glob_match(pattern: &str, text: &str) -> bool {
    let mut pi = pattern.chars().peekable();
    let mut ti = text.chars().peekable();

    while pi.peek().is_some() {
        match pi.peek() {
            Some('*') => {
                pi.next();
                if pi.peek().is_none() {
                    return true;
                }
                while ti.peek().is_some() {
                    let remaining_pattern: String = pi.clone().collect();
                    let remaining_text: String = ti.clone().collect();
                    if glob_match(&remaining_pattern, &remaining_text) {
                        return true;
                    }
                    ti.next();
                }
                return false;
            }
            Some('[') => {
                pi.next(); // skip '['
                let negate = pi.peek() == Some(&'!') || pi.peek() == Some(&'^');
                if negate {
                    pi.next();
                }

                let tc = match ti.next() {
                    Some(c) => c,
                    None => return false,
                };

                let mut matched = false;
                let mut prev_char = None;

                while let Some(&c) = pi.peek() {
                    if c == ']' {
                        pi.next();
                        break;
                    }
                    pi.next();

                    if c == '-' {
                        if let (Some(start), Some(&end)) = (prev_char, pi.peek()) {
                            pi.next();
                            if tc >= start && tc <= end {
                                matched = true;
                            }
                            prev_char = Some(end);
                            continue;
                        }
                    }

                    if tc == c {
                        matched = true;
                    }
                    prev_char = Some(c);
                }

                if negate {
                    matched = !matched;
                }
                if !matched {
                    return false;
                }
            }
            Some('?') => {
                pi.next();
                if ti.next().is_none() {
                    return false;
                }
            }
            Some(&pc) => {
                pi.next();
                match ti.next() {
                    Some(tc) if tc == pc => {}
                    _ => return false,
                }
            }
            None => break,
        }
    }

    ti.peek().is_none()
}
