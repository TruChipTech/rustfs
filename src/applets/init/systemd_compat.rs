/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! systemd-compatible init implementation for RustFS
//!
//! Parses .service unit files and manages services. Supports a subset of
//! systemd functionality sufficient for booting a Linux system:
//! - Type=simple, forking, oneshot
//! - ExecStart, ExecStartPre, ExecStartPost, ExecStop
//! - Restart=always, on-failure, no
//! - WantedBy, RequiredBy (for target resolution)
//! - After, Before, Requires, Wants (dependency ordering)

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const UNIT_PATHS: &[&str] = &[
    "/etc/systemd/system",
    "/usr/lib/systemd/system",
    "/lib/systemd/system",
];

/// Service type
#[derive(Debug, Clone, PartialEq)]
enum ServiceType {
    Simple,
    Forking,
    Oneshot,
}

/// Restart policy
#[derive(Debug, Clone, PartialEq)]
enum RestartPolicy {
    No,
    Always,
    OnFailure,
}

/// A parsed .service unit file
#[derive(Debug, Clone)]
struct ServiceUnit {
    name: String,
    description: String,
    service_type: ServiceType,
    exec_start_pre: Vec<String>,
    exec_start: String,
    exec_start_post: Vec<String>,
    exec_stop: String,
    restart: RestartPolicy,
    restart_sec: u64,
    working_directory: String,
    environment: Vec<(String, String)>,
    after: Vec<String>,
    before: Vec<String>,
    requires: Vec<String>,
    wants: Vec<String>,
    wanted_by: Vec<String>,
}

/// Runtime state of a service
struct ServiceState {
    unit: ServiceUnit,
    pid: Option<i32>,
    active: bool,
    restart_count: u32,
}

impl Default for ServiceUnit {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            service_type: ServiceType::Simple,
            exec_start_pre: Vec::new(),
            exec_start: String::new(),
            exec_start_post: Vec::new(),
            exec_stop: String::new(),
            restart: RestartPolicy::No,
            restart_sec: 1,
            working_directory: "/".to_string(),
            environment: Vec::new(),
            after: Vec::new(),
            before: Vec::new(),
            requires: Vec::new(),
            wants: Vec::new(),
            wanted_by: Vec::new(),
        }
    }
}

pub fn run(args: &[String]) -> i32 {
    let pid = unsafe { libc::getpid() };

    if pid != 1 {
        return handle_systemctl(args);
    }

    eprintln!("rustfs init: systemd-compatible init starting (PID 1)");

    // Set up signal handlers
    setup_signals();

    // Mount essential filesystems
    mount_essential_fs();

    // Discover and parse all service units
    let units = discover_units();
    eprintln!("init: discovered {} service units", units.len());

    // Determine target
    let target = if args.iter().any(|a| a.starts_with("--target=")) {
        args.iter()
            .find(|a| a.starts_with("--target="))
            .map(|a| a.trim_start_matches("--target=").to_string())
            .unwrap()
    } else {
        "multi-user.target".to_string()
    };

    eprintln!("init: reaching target: {target}");

    // Resolve which services to start for this target
    let ordered = resolve_target(&units, &target);

    // Start services in dependency order
    let mut states: HashMap<String, ServiceState> = HashMap::new();

    for unit_name in &ordered {
        if let Some(unit) = units.get(unit_name) {
            eprintln!("init: starting {}", unit.name);
            let pid = start_service(unit);
            states.insert(
                unit.name.clone(),
                ServiceState {
                    unit: unit.clone(),
                    pid,
                    active: pid.is_some() || unit.service_type == ServiceType::Oneshot,
                    restart_count: 0,
                },
            );
        }
    }

    // Main loop: reap children, restart services as needed
    loop {
        let mut status: libc::c_int = 0;
        let pid = unsafe { libc::waitpid(-1, &mut status, 0) };

        if pid > 0 {
            let exited_ok = libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0;

            let mut to_restart = None;
            for (name, state) in &states {
                if state.pid == Some(pid) {
                    eprintln!("init: service {} (pid {pid}) exited", name);
                    to_restart = Some(name.clone());
                    break;
                }
            }

            if let Some(name) = to_restart {
                if let Some(state) = states.get_mut(&name) {
                    state.active = false;
                    state.pid = None;

                    let should_restart = match state.unit.restart {
                        RestartPolicy::Always => true,
                        RestartPolicy::OnFailure => !exited_ok,
                        RestartPolicy::No => false,
                    };

                    if should_restart {
                        state.restart_count += 1;
                        if state.restart_count > 10 {
                            eprintln!("init: {} restarting too fast, pausing 10s", name);
                            std::thread::sleep(std::time::Duration::from_secs(10));
                            state.restart_count = 0;
                        }

                        let delay = state.unit.restart_sec;
                        if delay > 0 {
                            std::thread::sleep(std::time::Duration::from_secs(delay));
                        }

                        eprintln!("init: restarting {}", name);
                        state.pid = start_service(&state.unit);
                        state.active = state.pid.is_some();
                    }
                }
            }
        }
    }
}

/// Handle systemctl-like commands when not PID 1
fn handle_systemctl(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: init --systemd [--target=TARGET]");
        eprintln!("       init start|stop|restart|status <service>");
        return 1;
    }

    match args[0].as_str() {
        "poweroff" | "0" => {
            eprintln!("init: requesting poweroff...");
            unsafe { libc::kill(1, libc::SIGUSR1) };
            0
        }
        "reboot" | "6" => {
            eprintln!("init: requesting reboot...");
            unsafe { libc::kill(1, libc::SIGUSR2) };
            0
        }
        _ => {
            eprintln!("init: unsupported command: {}", args[0]);
            1
        }
    }
}

fn setup_signals() {
    unsafe {
        libc::signal(libc::SIGTSTP, libc::SIG_IGN);
        libc::signal(libc::SIGSTOP, libc::SIG_IGN);
        libc::signal(libc::SIGCHLD, libc::SIG_DFL);
    }
}

/// Mount essential pseudo-filesystems using direct syscalls
fn mount_essential_fs() {
    let mounts: &[(&str, &str, &str, libc::c_ulong, &str)] = &[
        ("proc", "/proc", "proc", 0, ""),
        ("sysfs", "/sys", "sysfs", 0, ""),
        ("devtmpfs", "/dev", "devtmpfs", 0, ""),
        (
            "tmpfs",
            "/run",
            "tmpfs",
            libc::MS_NOSUID | libc::MS_NODEV,
            "mode=0755",
        ),
        ("devpts", "/dev/pts", "devpts", 0, ""),
        (
            "tmpfs",
            "/dev/shm",
            "tmpfs",
            libc::MS_NOSUID | libc::MS_NODEV,
            "mode=1777",
        ),
        ("cgroup2", "/sys/fs/cgroup", "cgroup2", 0, ""),
    ];

    for (source, target, fstype, flags, opts) in mounts {
        if !is_mounted(target) {
            let _ = fs::create_dir_all(target);
            let c_source = std::ffi::CString::new(*source).unwrap();
            let c_target = std::ffi::CString::new(*target).unwrap();
            let c_fstype = std::ffi::CString::new(*fstype).unwrap();
            let c_opts = std::ffi::CString::new(*opts).unwrap();
            let data = if opts.is_empty() {
                std::ptr::null()
            } else {
                c_opts.as_ptr() as *const libc::c_void
            };
            let ret = unsafe {
                libc::mount(
                    c_source.as_ptr(),
                    c_target.as_ptr(),
                    c_fstype.as_ptr(),
                    *flags,
                    data,
                )
            };
            if ret == 0 {
                eprintln!("init: mounted {fstype} on {target}");
            }
        }
    }
}

fn is_mounted(path: &str) -> bool {
    if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
        mounts.lines().any(|line| {
            line.split_whitespace()
                .nth(1)
                .map_or(false, |mp| mp == path)
        })
    } else {
        false
    }
}

/// Discover all .service unit files from standard paths
fn discover_units() -> HashMap<String, ServiceUnit> {
    let mut units = HashMap::new();

    for dir in UNIT_PATHS {
        let dir_path = Path::new(dir);
        if !dir_path.is_dir() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("service") {
                    if let Some(unit) = parse_service_file(&path) {
                        // Don't override higher-priority paths
                        if !units.contains_key(&unit.name) {
                            units.insert(unit.name.clone(), unit);
                        }
                    }
                }
            }
        }
    }

    units
}

/// Parse a .service unit file
fn parse_service_file(path: &PathBuf) -> Option<ServiceUnit> {
    let content = fs::read_to_string(path).ok()?;
    let name = path.file_name()?.to_str()?.to_string();

    let mut unit = ServiceUnit::default();
    unit.name = name;

    let mut section = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].to_string();
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match section.as_str() {
                "Unit" => match key {
                    "Description" => unit.description = value.to_string(),
                    "After" => {
                        unit.after.extend(value.split_whitespace().map(String::from));
                    }
                    "Before" => {
                        unit.before.extend(value.split_whitespace().map(String::from));
                    }
                    "Requires" => {
                        unit.requires.extend(value.split_whitespace().map(String::from));
                    }
                    "Wants" => {
                        unit.wants.extend(value.split_whitespace().map(String::from));
                    }
                    _ => {}
                },
                "Service" => match key {
                    "Type" => {
                        unit.service_type = match value {
                            "forking" => ServiceType::Forking,
                            "oneshot" => ServiceType::Oneshot,
                            _ => ServiceType::Simple,
                        };
                    }
                    "ExecStartPre" => {
                        unit.exec_start_pre.push(value.to_string());
                    }
                    "ExecStart" => unit.exec_start = value.to_string(),
                    "ExecStartPost" => {
                        unit.exec_start_post.push(value.to_string());
                    }
                    "ExecStop" => unit.exec_stop = value.to_string(),
                    "Restart" => {
                        unit.restart = match value {
                            "always" => RestartPolicy::Always,
                            "on-failure" => RestartPolicy::OnFailure,
                            _ => RestartPolicy::No,
                        };
                    }
                    "RestartSec" => {
                        unit.restart_sec = value.parse().unwrap_or(1);
                    }
                    "WorkingDirectory" => {
                        unit.working_directory = value.to_string();
                    }
                    "Environment" => {
                        // Environment=KEY=VALUE
                        if let Some((k, v)) = value.split_once('=') {
                            unit.environment.push((
                                k.trim_matches('"').to_string(),
                                v.trim_matches('"').to_string(),
                            ));
                        }
                    }
                    _ => {}
                },
                "Install" => match key {
                    "WantedBy" => {
                        unit.wanted_by.extend(value.split_whitespace().map(String::from));
                    }
                    "RequiredBy" => {
                        // Treat same as WantedBy for ordering
                        unit.wanted_by.extend(value.split_whitespace().map(String::from));
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    if unit.exec_start.is_empty() {
        eprintln!("init: {}: no ExecStart, skipping", path.display());
        return None;
    }

    Some(unit)
}

/// Resolve which services should be started for a target, in dependency order
fn resolve_target(units: &HashMap<String, ServiceUnit>, target: &str) -> Vec<String> {
    let mut to_start: Vec<String> = Vec::new();
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Find all services that WantedBy this target
    let target_services: Vec<String> = units
        .values()
        .filter(|u| u.wanted_by.iter().any(|t| t == target))
        .map(|u| u.name.clone())
        .collect();

    // Topological sort based on After/Requires dependencies
    for name in &target_services {
        resolve_deps(name, units, &mut to_start, &mut visited);
    }

    to_start
}

/// Recursively resolve dependencies (depth-first topological sort)
fn resolve_deps(
    name: &str,
    units: &HashMap<String, ServiceUnit>,
    ordered: &mut Vec<String>,
    visited: &mut std::collections::HashSet<String>,
) {
    if visited.contains(name) {
        return;
    }
    visited.insert(name.to_string());

    if let Some(unit) = units.get(name) {
        // Resolve After dependencies first
        for dep in &unit.after {
            if units.contains_key(dep) {
                resolve_deps(dep, units, ordered, visited);
            }
        }
        // Resolve Requires dependencies
        for dep in &unit.requires {
            if units.contains_key(dep) {
                resolve_deps(dep, units, ordered, visited);
            }
        }
        // Resolve Wants dependencies
        for dep in &unit.wants {
            if units.contains_key(dep) {
                resolve_deps(dep, units, ordered, visited);
            }
        }
    }

    ordered.push(name.to_string());
}

/// Start a service unit, returning PID if applicable
fn start_service(unit: &ServiceUnit) -> Option<i32> {
    // Run ExecStartPre commands
    for pre_cmd in &unit.exec_start_pre {
        let cmd = pre_cmd.trim_start_matches('-'); // '-' prefix means ignore failure
        let ignore_fail = pre_cmd.starts_with('-');
        let status = run_service_command(cmd, unit);
        if !ignore_fail {
            if let Some(false) = status {
                eprintln!("init: {}: ExecStartPre failed: {cmd}", unit.name);
                return None;
            }
        }
    }

    // Run ExecStart
    match unit.service_type {
        ServiceType::Simple => {
            let pid = spawn_service_process(&unit.exec_start, unit);
            // Run ExecStartPost
            for post_cmd in &unit.exec_start_post {
                run_service_command(post_cmd, unit);
            }
            pid
        }
        ServiceType::Forking => {
            // Run and wait — the process itself forks
            run_service_command(&unit.exec_start, unit);
            for post_cmd in &unit.exec_start_post {
                run_service_command(post_cmd, unit);
            }
            None // PID tracking not supported for forking type
        }
        ServiceType::Oneshot => {
            run_service_command(&unit.exec_start, unit);
            for post_cmd in &unit.exec_start_post {
                run_service_command(post_cmd, unit);
            }
            None
        }
    }
}

/// Parse a command string into program and arguments
fn parse_command(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    for ch in cmd.chars() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }
        match ch {
            '\\' if !in_single_quote => escape_next = true,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

/// Check if a file is a script and return the interpreter
fn get_script_interpreter(path: &str) -> Option<Vec<String>> {
    if let Ok(content) = fs::read(path) {
        if content.starts_with(b"#!") {
            if let Some(line_end) = content.iter().position(|&b| b == b'\n') {
                let shebang = String::from_utf8_lossy(&content[2..line_end]).trim().to_string();
                let parts = parse_command(&shebang);
                if !parts.is_empty() {
                    return Some(parts);
                }
            }
        }
    }
    None
}

/// Run a command synchronously, returning Some(true) on success
fn run_service_command(cmd: &str, unit: &ServiceUnit) -> Option<bool> {
    let args = parse_command(cmd);
    if args.is_empty() {
        return Some(false);
    }

    let (program, cmd_args) = if let Some(mut interp) = get_script_interpreter(&args[0]) {
        let prog = interp.remove(0);
        interp.extend(args);
        (prog, interp)
    } else {
        let prog = args[0].clone();
        (prog, args[1..].to_vec())
    };

    let mut command = Command::new(&program);
    command.args(&cmd_args);
    command.current_dir(&unit.working_directory);

    for (key, val) in &unit.environment {
        command.env(key, val);
    }

    match command.status() {
        Ok(s) => Some(s.success()),
        Err(e) => {
            eprintln!("init: {}: failed to execute: {e}", unit.name);
            Some(false)
        }
    }
}

/// Spawn a service process in the background
fn spawn_service_process(cmd: &str, unit: &ServiceUnit) -> Option<i32> {
    let args = parse_command(cmd);
    if args.is_empty() {
        return None;
    }

    let (program, cmd_args) = if let Some(mut interp) = get_script_interpreter(&args[0]) {
        let prog = interp.remove(0);
        interp.extend(args);
        (prog, interp)
    } else {
        let prog = args[0].clone();
        (prog, args[1..].to_vec())
    };

    let mut command = Command::new(&program);
    command.args(&cmd_args);
    command.current_dir(&unit.working_directory);

    for (key, val) in &unit.environment {
        command.env(key, val);
    }

    match command.spawn() {
        Ok(child) => Some(child.id() as i32),
        Err(e) => {
            eprintln!("init: {}: failed to spawn: {e}", unit.name);
            None
        }
    }
}
