/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! System V init implementation for RustFS
//!
//! Parses /etc/inittab and manages runlevels, process spawning/respawning,
//! and system lifecycle (boot → runlevel → shutdown/reboot).

use std::collections::HashMap;
use std::fs;
use std::process::Command;

/// Inittab action types
#[derive(Debug, Clone, PartialEq)]
enum Action {
    Sysinit,
    Boot,
    Bootwait,
    Wait,
    Once,
    Respawn,
    Ctrlaltdel,
    Shutdown,
    Restart,
    Initdefault,
    Powerwait,
}

/// A parsed inittab entry
#[derive(Debug, Clone)]
struct InittabEntry {
    id: String,
    runlevels: Vec<u8>,
    action: Action,
    process: String,
}

/// State of a managed process
struct ProcessState {
    pid: Option<i32>,
    entry: InittabEntry,
    restart_count: u32,
}

pub fn run(args: &[String]) -> i32 {
    let pid = unsafe { libc::getpid() };

    if pid != 1 {
        // Not running as PID 1 — maybe user wants to send a signal to init
        return handle_telinit(args);
    }

    eprintln!("rustfs init: System V init starting (PID 1)");

    // Set up signal handlers
    setup_signals();

    // Mount essential filesystems
    mount_essential_fs();

    // Set hostname early (before any getty/login)
    set_hostname();

    // Parse inittab
    let inittab_path = if args.iter().any(|a| a.starts_with("--inittab=")) {
        args.iter()
            .find(|a| a.starts_with("--inittab="))
            .map(|a| a.trim_start_matches("--inittab=").to_string())
            .unwrap()
    } else {
        "/etc/inittab".to_string()
    };

    let entries = match parse_inittab(&inittab_path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("init: failed to parse {inittab_path}: {e}");
            eprintln!("init: entering emergency shell");
            spawn_shell();
            return 1;
        }
    };

    // Determine default runlevel
    let default_runlevel = get_default_runlevel(&entries, args);
    eprintln!("init: default runlevel: {default_runlevel}");

    // Execute sysinit entries
    run_action_entries(&entries, Action::Sysinit, 0);

    // Execute boot entries
    run_action_entries(&entries, Action::Boot, 0);
    run_action_entries(&entries, Action::Bootwait, 0);

    // Enter default runlevel
    enter_runlevel(&entries, default_runlevel);

    0
}

/// Handle telinit commands (when not PID 1)
fn handle_telinit(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: init <runlevel>");
        eprintln!("       init {{0|1|2|3|4|5|6|s|S|q|Q}}");
        return 1;
    }

    let rl = &args[0];
    match rl.as_str() {
        "0" => {
            eprintln!("init: requesting shutdown...");
            unsafe { libc::kill(1, libc::SIGUSR1) };
        }
        "6" => {
            eprintln!("init: requesting reboot...");
            unsafe { libc::kill(1, libc::SIGUSR2) };
        }
        "q" | "Q" => {
            eprintln!("init: re-reading /etc/inittab...");
            unsafe { libc::kill(1, libc::SIGHUP) };
        }
        _ => {
            eprintln!("init: switching to runlevel {rl}");
            // In a full implementation, would use a control socket
            // For now, just send SIGHUP to re-read inittab
            unsafe { libc::kill(1, libc::SIGHUP) };
        }
    }
    0
}

/// Set up signal handlers for init
fn setup_signals() {
    unsafe {
        // Ignore signals that shouldn't kill init
        libc::signal(libc::SIGTSTP, libc::SIG_IGN);
        libc::signal(libc::SIGSTOP, libc::SIG_IGN);

        // SIGCHLD — reap children (handled in main loop)
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

/// Check if a path is already a mount point
fn is_mounted(path: &str) -> bool {
    if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
        mounts.lines().any(|line| {
            line.split_whitespace()
                .nth(1) == Some(path)
        })
    } else {
        false
    }
}

/// Set hostname from /etc/hostname
fn set_hostname() {
    if let Ok(name) = fs::read_to_string("/etc/hostname") {
        let name = name.trim();
        if !name.is_empty() {
            if let Ok(c_name) = std::ffi::CString::new(name) {
                unsafe { libc::sethostname(c_name.as_ptr(), name.len()) };
            }
        }
    }
}

/// Parse /etc/inittab into entries
fn parse_inittab(path: &str) -> Result<Vec<InittabEntry>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("{e}"))?;

    let mut entries = Vec::new();

    for (lineno, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.splitn(4, ':').collect();
        if parts.len() != 4 {
            eprintln!("init: {path}:{}: malformed line, skipping", lineno + 1);
            continue;
        }

        let id = parts[0].to_string();
        let runlevels: Vec<u8> = parts[1]
            .chars()
            .filter_map(|c| match c {
                '0'..='9' => Some(c as u8 - b'0'),
                's' | 'S' => Some(b'S' - b'0'),
                _ => None,
            })
            .collect();

        let action = match parts[2] {
            "sysinit" => Action::Sysinit,
            "boot" => Action::Boot,
            "bootwait" => Action::Bootwait,
            "wait" => Action::Wait,
            "once" => Action::Once,
            "respawn" => Action::Respawn,
            "ctrlaltdel" => Action::Ctrlaltdel,
            "shutdown" => Action::Shutdown,
            "restart" => Action::Restart,
            "initdefault" => Action::Initdefault,
            "powerwait" => Action::Powerwait,
            other => {
                eprintln!("init: {path}:{}: unknown action '{other}', skipping", lineno + 1);
                continue;
            }
        };

        let process = parts[3].to_string();

        entries.push(InittabEntry {
            id,
            runlevels,
            action,
            process,
        });
    }

    if entries.is_empty() {
        return Err("no valid entries found".to_string());
    }

    Ok(entries)
}

/// Get the default runlevel from inittab or command line
fn get_default_runlevel(entries: &[InittabEntry], args: &[String]) -> u8 {
    // Check command line first (kernel passes runlevel as arg)
    for arg in args {
        if let Ok(rl) = arg.parse::<u8>() {
            if rl <= 6 {
                return rl;
            }
        }
    }

    // Look for initdefault in inittab
    for entry in entries {
        if entry.action == Action::Initdefault && !entry.runlevels.is_empty() {
            return entry.runlevels[0];
        }
    }

    // Default to runlevel 3 (multi-user with networking)
    3
}

/// Run all entries matching a specific action (for non-runlevel actions like sysinit)
fn run_action_entries(entries: &[InittabEntry], action: Action, _runlevel: u8) {
    for entry in entries {
        if entry.action == action {
            eprintln!("init: executing [{}]: {}", entry.id, entry.process);
            run_command_wait(&entry.process);
        }
    }
}

/// Enter a runlevel: execute wait/once/respawn entries for that runlevel
fn enter_runlevel(entries: &[InittabEntry], runlevel: u8) {
    eprintln!("init: entering runlevel {runlevel}");

    let mut respawn_procs: HashMap<String, ProcessState> = HashMap::new();

    // Run 'wait' entries first (blocking)
    for entry in entries {
        if entry.action == Action::Wait && entry.runlevels.contains(&runlevel) {
            eprintln!("init: [{}] wait: {}", entry.id, entry.process);
            run_command_wait(&entry.process);
        }
    }

    // Run 'once' entries (non-blocking)
    for entry in entries {
        if entry.action == Action::Once && entry.runlevels.contains(&runlevel) {
            eprintln!("init: [{}] once: {}", entry.id, entry.process);
            if let Some(pid) = spawn_process(&entry.process) {
                eprintln!("init: [{}] started (pid {pid})", entry.id);
            }
        }
    }

    // Start 'respawn' entries and enter main loop
    for entry in entries {
        if entry.action == Action::Respawn && entry.runlevels.contains(&runlevel) {
            let pid = spawn_process(&entry.process);
            eprintln!("init: [{}] respawn: {} (pid {:?})", entry.id, entry.process, pid);
            respawn_procs.insert(
                entry.id.clone(),
                ProcessState {
                    pid,
                    entry: entry.clone(),
                    restart_count: 0,
                },
            );
        }
    }

    // Main loop: reap children and respawn as needed
    loop {
        let mut status: libc::c_int = 0;
        let pid = unsafe { libc::waitpid(-1, &mut status, 0) };

        if pid > 0 {
            // Check if this was a respawn process
            let mut to_respawn = None;
            for (id, state) in &respawn_procs {
                if state.pid == Some(pid) {
                    eprintln!("init: [{}] process (pid {pid}) exited", id);
                    to_respawn = Some(id.clone());
                    break;
                }
            }

            if let Some(id) = to_respawn {
                if let Some(state) = respawn_procs.get_mut(&id) {
                    // Rate-limit respawning (max 10 restarts, then pause)
                    state.restart_count += 1;
                    if state.restart_count > 10 {
                        eprintln!("init: [{}] respawning too fast, pausing 5s", id);
                        std::thread::sleep(std::time::Duration::from_secs(5));
                        state.restart_count = 0;
                    }

                    let new_pid = spawn_process(&state.entry.process);
                    eprintln!("init: [{}] respawned (pid {:?})", id, new_pid);
                    state.pid = new_pid;
                }
            }
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

/// Run a command and wait for completion (executes directly)
fn run_command_wait(cmd: &str) {
    let args = parse_command(cmd);
    if args.is_empty() {
        return;
    }

    let (program, cmd_args) = if let Some(mut interp) = get_script_interpreter(&args[0]) {
        let prog = interp.remove(0);
        interp.extend(args);
        (prog, interp)
    } else {
        let prog = args[0].clone();
        (prog, args[1..].to_vec())
    };

    match Command::new(&program).args(&cmd_args).status() {
        Ok(s) => {
            if !s.success() {
                eprintln!("init: command failed (exit {:?}): {cmd}", s.code());
            }
        }
        Err(e) => {
            eprintln!("init: failed to execute: {cmd}: {e}");
        }
    }
}

/// Spawn a process in the background, returning its PID
fn spawn_process(cmd: &str) -> Option<i32> {
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

    match Command::new(&program).args(&cmd_args).spawn() {
        Ok(child) => Some(child.id() as i32),
        Err(e) => {
            eprintln!("init: failed to spawn: {cmd}: {e}");
            None
        }
    }
}

/// Spawn an emergency shell
fn spawn_shell() {
    let shells = ["/bin/sh", "/bin/bash", "/bin/ash"];
    for shell in &shells {
        if std::path::Path::new(shell).exists() {
            eprintln!("init: spawning emergency shell: {shell}");
            let _ = Command::new(shell).status();
            return;
        }
    }
    eprintln!("init: no shell found!");
}
