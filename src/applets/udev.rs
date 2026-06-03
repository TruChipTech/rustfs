/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! udev — udev-compatible device manager for RustFS
//!
//! Listens for kernel uevents via netlink and creates/removes device nodes
//! in /dev. Reads rules from /etc/udev/rules.d/ and /usr/lib/udev/rules.d/.
//!
//! Supported rule keys:
//!   Match:  SUBSYSTEM, KERNEL, ATTR{...}, ENV{...}, ACTION, DEVPATH
//!   Assign: NAME, SYMLINK, MODE, OWNER, GROUP, RUN, ENV{...}
//!
//! Usage:
//!   udevd                   Run as daemon
//!   udevd --scan            Coldplug: trigger events for existing devices
//!   udevadm trigger         Trigger events for existing devices
//!   udevadm info <device>   Show device info

use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::Command;

const RULES_DIRS: &[&str] = &[
    "/etc/udev/rules.d",
    "/usr/lib/udev/rules.d",
    "/lib/udev/rules.d",
];

/// A single key=value or key==value match/assignment in a rule line
#[derive(Debug, Clone)]
enum RuleToken {
    // Match operators (==)
    MatchSubsystem(String),
    MatchKernel(String),
    MatchAction(String),
    MatchDevpath(String),
    MatchAttr(String, String),  // ATTR{key}==value
    MatchEnv(String, String),   // ENV{key}==value

    // Assignment operators (=, +=)
    AssignName(String),
    AssignSymlink(String),
    AssignMode(String),
    AssignOwner(String),
    AssignGroup(String),
    AssignRun(String),
    AssignEnv(String, String),  // ENV{key}=value
}

/// A parsed udev rule (one line of a rules file)
#[derive(Debug, Clone)]
struct UdevRule {
    tokens: Vec<RuleToken>,
    #[allow(dead_code)]
    source_file: String,
}

/// Event data from kernel uevent
#[derive(Debug, Default)]
struct UEvent {
    action: String,
    devpath: String,
    subsystem: String,
    devname: String,
    devtype: String,
    major: String,
    minor: String,
    seqnum: String,
    properties: HashMap<String, String>,
}

pub fn run(args: &[String]) -> i32 {
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("");

    match cmd {
        "--scan" | "trigger" => return trigger_coldplug(),
        "info" => {
            if args.len() < 2 {
                eprintln!("Usage: udevadm info <device_path>");
                return 1;
            }
            return show_device_info(&args[1]);
        }
        _ => {}
    }

    // Default: run as daemon
    run_daemon()
}

/// Run as a daemon, listening for kernel uevents
fn run_daemon() -> i32 {
    eprintln!("udevd: starting");

    let rules = load_all_rules();
    eprintln!("udevd: loaded {} rules", rules.len());

    // Create netlink socket for kernel uevents
    let sock = unsafe {
        let s = libc::socket(libc::AF_NETLINK, libc::SOCK_DGRAM, 15 /* NETLINK_KOBJECT_UEVENT */);
        if s < 0 {
            eprintln!("udevd: failed to create netlink socket");
            return 1;
        }

        let mut addr: libc::sockaddr_nl = std::mem::zeroed();
        addr.nl_family = libc::AF_NETLINK as u16;
        addr.nl_groups = 1;
        addr.nl_pid = libc::getpid() as u32;

        let ret = libc::bind(
            s,
            &addr as *const libc::sockaddr_nl as *const libc::sockaddr,
            std::mem::size_of::<libc::sockaddr_nl>() as u32,
        );
        if ret < 0 {
            eprintln!("udevd: failed to bind netlink socket");
            libc::close(s);
            return 1;
        }
        s
    };

    eprintln!("udevd: listening for uevents");

    let mut buf = vec![0u8; 16384];
    loop {
        let len = unsafe {
            libc::recv(sock, buf.as_mut_ptr() as *mut libc::c_void, buf.len(), 0)
        };
        if len <= 0 {
            continue;
        }

        let event = parse_uevent(&buf[..len as usize]);
        if !event.action.is_empty() {
            process_event(&event, &rules);
        }
    }
}

/// Parse a kernel uevent message (null-delimited key=value pairs)
fn parse_uevent(data: &[u8]) -> UEvent {
    let mut event = UEvent::default();

    let msg = String::from_utf8_lossy(data);
    for part in msg.split('\0') {
        if part.is_empty() {
            continue;
        }
        // Skip the initial summary line (e.g. "add@/devices/...")
        if part.contains('@') && !part.contains('=') {
            if let Some((action, path)) = part.split_once('@') {
                event.action = action.to_string();
                event.devpath = path.to_string();
            }
            continue;
        }
        if let Some((key, value)) = part.split_once('=') {
            match key {
                "ACTION" => event.action = value.to_string(),
                "DEVPATH" => event.devpath = value.to_string(),
                "SUBSYSTEM" => event.subsystem = value.to_string(),
                "DEVNAME" => event.devname = value.to_string(),
                "DEVTYPE" => event.devtype = value.to_string(),
                "MAJOR" => event.major = value.to_string(),
                "MINOR" => event.minor = value.to_string(),
                "SEQNUM" => event.seqnum = value.to_string(),
                _ => {
                    event.properties.insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    event
}

/// Process a uevent against the loaded rules
fn process_event(event: &UEvent, rules: &[UdevRule]) {
    let mut name = event.devname.clone();
    let mut symlinks: Vec<String> = Vec::new();
    let mut mode = 0o660u32;
    let mut owner = 0u32;
    let mut group = 0u32;
    let mut run_cmds: Vec<String> = Vec::new();
    let mut env_assigns: Vec<(String, String)> = Vec::new();

    // Evaluate rules in order
    for rule in rules {
        if rule_matches(rule, event) {
            // Apply assignments
            for token in &rule.tokens {
                match token {
                    RuleToken::AssignName(n) => {
                        name = substitute_vars(n, event);
                    }
                    RuleToken::AssignSymlink(s) => {
                        symlinks.push(substitute_vars(s, event));
                    }
                    RuleToken::AssignMode(m) => {
                        mode = u32::from_str_radix(m, 8).unwrap_or(0o660);
                    }
                    RuleToken::AssignOwner(o) => {
                        owner = o.parse().unwrap_or(0);
                    }
                    RuleToken::AssignGroup(g) => {
                        group = g.parse().unwrap_or(0);
                    }
                    RuleToken::AssignRun(r) => {
                        run_cmds.push(substitute_vars(r, event));
                    }
                    RuleToken::AssignEnv(k, v) => {
                        env_assigns.push((k.clone(), substitute_vars(v, event)));
                    }
                    _ => {} // match tokens already evaluated
                }
            }
        }
    }

    // If no name was determined, use devname or derive from devpath
    if name.is_empty() {
        name = event
            .devpath
            .rsplit('/')
            .next()
            .unwrap_or("")
            .to_string();
    }

    if name.is_empty() {
        return;
    }

    match event.action.as_str() {
        "add" | "change" => {
            if !event.major.is_empty() && !event.minor.is_empty() {
                let dev_path = format!("/dev/{name}");
                let major: u32 = event.major.parse().unwrap_or(0);
                let minor: u32 = event.minor.parse().unwrap_or(0);

                if !Path::new(&dev_path).exists() {
                    let dev = libc::makedev(major, minor);
                    let node_type = if event.subsystem == "block" {
                        libc::S_IFBLK
                    } else {
                        libc::S_IFCHR
                    };

                    if let Ok(c_path) = std::ffi::CString::new(dev_path.as_str()) {
                        // Ensure parent directory exists
                        if let Some(parent) = Path::new(&dev_path).parent() {
                            let _ = fs::create_dir_all(parent);
                        }

                        let ret = unsafe { libc::mknod(c_path.as_ptr(), node_type | mode, dev) };
                        if ret == 0 {
                            unsafe { libc::chown(c_path.as_ptr(), owner, group) };
                        }
                    }
                }

                // Create symlinks
                for link in &symlinks {
                    let link_path = format!("/dev/{link}");
                    if let Some(parent) = Path::new(&link_path).parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let _ = fs::remove_file(&link_path); // remove old symlink
                    let _ = symlink(&dev_path, &link_path);
                }
            }
        }
        "remove" => {
            let dev_path = format!("/dev/{name}");
            let _ = fs::remove_file(&dev_path);
            for link in &symlinks {
                let link_path = format!("/dev/{link}");
                let _ = fs::remove_file(&link_path);
            }
        }
        _ => {}
    }

    // Set environment variables
    for (k, v) in &env_assigns {
        std::env::set_var(k, v);
    }

    // Run commands
    for cmd in &run_cmds {
        let _ = Command::new("/bin/sh")
            .args(["-c", cmd])
            .env("DEVNAME", &name)
            .env("ACTION", &event.action)
            .env("SUBSYSTEM", &event.subsystem)
            .env("DEVPATH", &event.devpath)
            .status();
    }
}

/// Check if all match tokens in a rule match the event
fn rule_matches(rule: &UdevRule, event: &UEvent) -> bool {
    for token in &rule.tokens {
        match token {
            RuleToken::MatchAction(pat) => {
                if !pattern_match(pat, &event.action) {
                    return false;
                }
            }
            RuleToken::MatchSubsystem(pat) => {
                if !pattern_match(pat, &event.subsystem) {
                    return false;
                }
            }
            RuleToken::MatchKernel(pat) => {
                let kernel_name = event
                    .devname
                    .split('/')
                    .last()
                    .unwrap_or(&event.devname);
                if !pattern_match(pat, kernel_name) {
                    return false;
                }
            }
            RuleToken::MatchDevpath(pat) => {
                if !pattern_match(pat, &event.devpath) {
                    return false;
                }
            }
            RuleToken::MatchAttr(key, pat) => {
                let attr_path = format!("/sys{}/{key}", event.devpath);
                let val = fs::read_to_string(&attr_path)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                if !pattern_match(pat, &val) {
                    return false;
                }
            }
            RuleToken::MatchEnv(key, pat) => {
                let val = event
                    .properties
                    .get(key)
                    .cloned()
                    .unwrap_or_default();
                if !pattern_match(pat, &val) {
                    return false;
                }
            }
            _ => {} // assignment tokens don't affect matching
        }
    }
    true
}

/// Simple glob pattern matching for udev patterns
fn pattern_match(pattern: &str, text: &str) -> bool {
    // Handle | (alternation) in patterns
    if pattern.contains('|') {
        return pattern.split('|').any(|p| glob_match(p.trim(), text));
    }
    glob_match(pattern, text)
}

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
                let remaining_pattern: String = pi.clone().collect();
                while ti.peek().is_some() {
                    let remaining_text: String = ti.clone().collect();
                    if glob_match(&remaining_pattern, &remaining_text) {
                        return true;
                    }
                    ti.next();
                }
                return glob_match(&remaining_pattern, "");
            }
            Some('?') => {
                pi.next();
                if ti.next().is_none() {
                    return false;
                }
            }
            Some('[') => {
                pi.next();
                let negate = pi.peek() == Some(&'!');
                if negate {
                    pi.next();
                }
                let tc = match ti.next() {
                    Some(c) => c,
                    None => return false,
                };
                let mut matched = false;
                let mut prev = None;
                while let Some(&c) = pi.peek() {
                    if c == ']' {
                        pi.next();
                        break;
                    }
                    pi.next();
                    if c == '-' {
                        if let (Some(start), Some(&end)) = (prev, pi.peek()) {
                            pi.next();
                            if tc >= start && tc <= end {
                                matched = true;
                            }
                            prev = Some(end);
                            continue;
                        }
                    }
                    if tc == c {
                        matched = true;
                    }
                    prev = Some(c);
                }
                if negate {
                    matched = !matched;
                }
                if !matched {
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

/// Substitute %k, %n, $kernel, $number, etc. in strings
fn substitute_vars(template: &str, event: &UEvent) -> String {
    let kernel_name = event
        .devpath
        .rsplit('/')
        .next()
        .unwrap_or("");

    let number = kernel_name
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();

    template
        .replace("%k", kernel_name)
        .replace("$kernel", kernel_name)
        .replace("%n", &number)
        .replace("$number", &number)
        .replace("%M", &event.major)
        .replace("$major", &event.major)
        .replace("%m", &event.minor)
        .replace("$minor", &event.minor)
        .replace("%s", &event.subsystem)
}

/// Load all rules from standard directories
fn load_all_rules() -> Vec<UdevRule> {
    let mut rules = Vec::new();
    let mut files: Vec<(String, String)> = Vec::new(); // (filename, full_path)

    for dir in RULES_DIRS {
        let dir_path = Path::new(dir);
        if !dir_path.is_dir() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("rules") {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        files.push((name.to_string(), path.to_string_lossy().to_string()));
                    }
                }
            }
        }
    }

    // Sort by filename (priority order: 00-xxx.rules before 99-xxx.rules)
    files.sort_by(|a, b| a.0.cmp(&b.0));

    for (_, path) in &files {
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some(rule) = parse_rule_line(line, path) {
                    rules.push(rule);
                }
            }
        }
    }

    rules
}

/// Parse a single udev rule line
fn parse_rule_line(line: &str, source_file: &str) -> Option<UdevRule> {
    let mut tokens = Vec::new();

    // Split by comma, handling quoted strings
    for part in split_rule_parts(line) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // Match operators: KEY==VALUE
        if let Some((key, value)) = part.split_once("==") {
            let key = key.trim();
            let value = value.trim().trim_matches('"');
            match key {
                "SUBSYSTEM" => tokens.push(RuleToken::MatchSubsystem(value.to_string())),
                "KERNEL" => tokens.push(RuleToken::MatchKernel(value.to_string())),
                "ACTION" => tokens.push(RuleToken::MatchAction(value.to_string())),
                "DEVPATH" => tokens.push(RuleToken::MatchDevpath(value.to_string())),
                k if k.starts_with("ATTR{") && k.ends_with('}') => {
                    let attr = &k[5..k.len() - 1];
                    tokens.push(RuleToken::MatchAttr(attr.to_string(), value.to_string()));
                }
                k if k.starts_with("ENV{") && k.ends_with('}') => {
                    let env_key = &k[4..k.len() - 1];
                    tokens.push(RuleToken::MatchEnv(env_key.to_string(), value.to_string()));
                }
                _ => {}
            }
        }
        // Assignment operators: KEY=VALUE or KEY+=VALUE
        else if let Some((key, value)) = part.split_once('=') {
            let key = key.trim().trim_end_matches('+');
            let value = value.trim().trim_matches('"');
            match key {
                "NAME" => tokens.push(RuleToken::AssignName(value.to_string())),
                "SYMLINK" => tokens.push(RuleToken::AssignSymlink(value.to_string())),
                "MODE" => tokens.push(RuleToken::AssignMode(value.to_string())),
                "OWNER" => tokens.push(RuleToken::AssignOwner(value.to_string())),
                "GROUP" => tokens.push(RuleToken::AssignGroup(value.to_string())),
                "RUN" => tokens.push(RuleToken::AssignRun(value.to_string())),
                k if k.starts_with("ENV{") && k.ends_with('}') => {
                    let env_key = &k[4..k.len() - 1];
                    tokens.push(RuleToken::AssignEnv(env_key.to_string(), value.to_string()));
                }
                _ => {}
            }
        }
    }

    if tokens.is_empty() {
        return None;
    }

    Some(UdevRule {
        tokens,
        source_file: source_file.to_string(),
    })
}

/// Split a rule line by commas, respecting quoted strings
fn split_rule_parts(line: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in line.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                current.push(c);
            }
            ',' if !in_quotes => {
                parts.push(current.clone());
                current.clear();
            }
            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

/// Trigger coldplug events for existing devices
fn trigger_coldplug() -> i32 {
    eprintln!("udevadm: triggering coldplug events");

    trigger_dir("/sys/class");
    trigger_dir("/sys/block");
    trigger_dir("/sys/devices");

    eprintln!("udevadm: coldplug complete");
    0
}

/// Write "add" to uevent files in sysfs to trigger coldplug
fn trigger_dir(base: &str) {
    let base_path = Path::new(base);
    if !base_path.is_dir() {
        return;
    }

    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            let uevent_path = entry.path().join("uevent");
            if uevent_path.exists() {
                let _ = fs::write(&uevent_path, "add");
            }

            // Recurse into subdirectories
            if entry.path().is_dir() {
                trigger_dir(&entry.path().to_string_lossy());
            }
        }
    }
}

/// Show device information from sysfs
fn show_device_info(device: &str) -> i32 {
    // Accept /dev/xxx or /sys/xxx paths
    let syspath = if device.starts_with("/dev/") {
        let devname = &device[5..];
        // Try to find in /sys/class
        let mut found = String::new();
        if let Ok(classes) = fs::read_dir("/sys/class") {
            for class in classes.flatten() {
                let check = class.path().join(devname);
                if check.is_dir() {
                    found = check.to_string_lossy().to_string();
                    break;
                }
            }
        }
        if found.is_empty() {
            // Try /sys/block
            let check = format!("/sys/block/{devname}");
            if Path::new(&check).is_dir() {
                found = check;
            }
        }
        found
    } else {
        device.to_string()
    };

    if syspath.is_empty() || !Path::new(&syspath).is_dir() {
        eprintln!("udevadm: device not found: {device}");
        return 1;
    }

    println!("P: {syspath}");

    // Read uevent
    let uevent_path = format!("{syspath}/uevent");
    if let Ok(content) = fs::read_to_string(&uevent_path) {
        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                println!("E: {key}={value}");
            }
        }
    }

    // Read dev (major:minor)
    let dev_path = format!("{syspath}/dev");
    if let Ok(dev) = fs::read_to_string(&dev_path) {
        println!("N: {}", dev.trim());
    }

    0
}
