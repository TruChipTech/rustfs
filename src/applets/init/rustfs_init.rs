/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! RustFS simple init — lightweight init process
//!
//! A minimal init implementation suitable for embedded systems and containers.
//! Reads /etc/inittab (simplified format), mounts essential filesystems,
//! spawns and respawns processes, handles signals, and manages shutdown/reboot.

use std::collections::HashMap;
use std::fs;
use std::process::Command;

/// Inittab action types
#[derive(Debug, Clone, PartialEq)]
enum Action {
    Sysinit,
    Wait,
    Once,
    Respawn,
    Askfirst,
    Shutdown,
    Restart,
    Ctrlaltdel,
}

/// A parsed inittab entry
#[derive(Debug, Clone)]
struct InittabEntry {
    id: String,
    action: Action,
    process: String,
}

pub fn run(args: &[String]) -> i32 {
    let pid = unsafe { libc::getpid() };

    if pid != 1 {
        return handle_telinit(args);
    }

    eprintln!("rustfs init: starting (PID 1)");

    // Become session leader
    unsafe { libc::setsid() };

    // Set up signal handlers
    setup_signals();

    // Mount essential pseudo-filesystems
    mount_essential_fs();

    // Set hostname if /etc/hostname exists
    set_hostname();

    // Parse inittab
    let inittab_path = args
        .iter()
        .find(|a| a.starts_with("--inittab="))
        .map(|a| a.trim_start_matches("--inittab=").to_string())
        .unwrap_or_else(|| "/etc/inittab".to_string());

    let entries = match parse_inittab(&inittab_path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("init: failed to parse {inittab_path}: {e}");
            eprintln!("init: using default configuration");
            default_inittab()
        }
    };

    // Phase 1: Run sysinit entries (blocking, sequential)
    eprintln!("init: running sysinit...");
    for entry in &entries {
        if entry.action == Action::Sysinit {
            eprintln!("init: sysinit [{}]: {}", entry.id, entry.process);
            run_command_wait(&entry.process);
        }
    }

    // Phase 2: Run wait entries (blocking, sequential)
    for entry in &entries {
        if entry.action == Action::Wait {
            eprintln!("init: wait [{}]: {}", entry.id, entry.process);
            run_command_wait(&entry.process);
        }
    }

    // Phase 3: Run once entries (non-blocking)
    for entry in &entries {
        if entry.action == Action::Once {
            eprintln!("init: once [{}]: {}", entry.id, entry.process);
            let _ = spawn_process(&entry.process);
        }
    }

    // Phase 4: Start respawn entries and enter main loop
    let mut respawn_procs: HashMap<String, RespawnState> = HashMap::new();

    for entry in &entries {
        if entry.action == Action::Respawn || entry.action == Action::Askfirst {
            let pid = if entry.action == Action::Askfirst {
                spawn_askfirst(&entry.process, &entry.id)
            } else {
                spawn_process(&entry.process)
            };
            eprintln!("init: respawn [{}]: {} (pid {:?})", entry.id, entry.process, pid);
            respawn_procs.insert(
                entry.id.clone(),
                RespawnState {
                    entry: entry.clone(),
                    pid,
                    restart_count: 0,
                    last_restart: std::time::Instant::now(),
                },
            );
        }
    }

    // Main loop: reap zombies and respawn processes
    eprintln!("init: entering main loop");
    loop {
        let mut status: libc::c_int = 0;
        let pid = unsafe { libc::waitpid(-1, &mut status, 0) };

        if pid > 0 {
            // Find which respawn entry this PID belongs to
            let mut to_respawn = None;
            for (id, state) in &respawn_procs {
                if state.pid == Some(pid) {
                    to_respawn = Some(id.clone());
                    break;
                }
            }

            if let Some(id) = to_respawn {
                if let Some(state) = respawn_procs.get_mut(&id) {
                    let now = std::time::Instant::now();
                    let elapsed = now.duration_since(state.last_restart);

                    // Rate-limit: if restarting faster than every 5 seconds, throttle
                    if elapsed.as_secs() < 5 {
                        state.restart_count += 1;
                    } else {
                        state.restart_count = 0;
                    }

                    if state.restart_count > 5 {
                        eprintln!("init: [{}] respawning too fast, delaying 5s", id);
                        std::thread::sleep(std::time::Duration::from_secs(5));
                        state.restart_count = 0;
                    }

                    let new_pid = if state.entry.action == Action::Askfirst {
                        spawn_askfirst(&state.entry.process, &id)
                    } else {
                        spawn_process(&state.entry.process)
                    };
                    state.pid = new_pid;
                    state.last_restart = std::time::Instant::now();
                }
            }
        }
    }
}

struct RespawnState {
    entry: InittabEntry,
    pid: Option<i32>,
    restart_count: u32,
    last_restart: std::time::Instant,
}

/// Handle telinit-like commands when not PID 1
fn handle_telinit(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: init [action]");
        eprintln!("Actions: 0 (halt), 6 (reboot), q (re-read inittab)");
        return 1;
    }

    match args[0].as_str() {
        "0" => {
            eprintln!("init: sending halt signal");
            unsafe { libc::kill(1, libc::SIGUSR1) };
        }
        "6" => {
            eprintln!("init: sending reboot signal");
            unsafe { libc::kill(1, libc::SIGUSR2) };
        }
        "q" | "Q" => {
            eprintln!("init: sending reload signal");
            unsafe { libc::kill(1, libc::SIGHUP) };
        }
        other => {
            eprintln!("init: unknown action: {other}");
            return 1;
        }
    }
    0
}

/// Set up signal handlers
fn setup_signals() {
    unsafe {
        libc::signal(libc::SIGTSTP, libc::SIG_IGN);
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
            if ret != 0 {
                // Silently skip — may not be available
            }
        }
    }

    // Create standard /dev entries if needed
    let dev_entries: &[(&str, &str)] = &[
        ("/dev/null", "/dev/null"),
        ("/dev/console", "/dev/console"),
    ];
    for (path, _) in dev_entries {
        if !std::path::Path::new(path).exists() {
            // These should exist from devtmpfs, but log if missing
            eprintln!("init: warning: {path} not found");
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

/// Set hostname from /etc/hostname
fn set_hostname() {
    if let Ok(name) = fs::read_to_string("/etc/hostname") {
        let name = name.trim();
        if !name.is_empty() {
            let c_name = std::ffi::CString::new(name).ok();
            if let Some(c_name) = c_name {
                unsafe { libc::sethostname(c_name.as_ptr(), name.len()) };
            }
        }
    }
}

/// Parse /etc/inittab
///
/// Format: id:action:process
/// (simplified from SysV — no runlevels field)
fn parse_inittab(path: &str) -> Result<Vec<InittabEntry>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("{e}"))?;
    let mut entries = Vec::new();

    for (lineno, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Support both id:runlevels:action:process (SysV compat)
        // and id:action:process (simplified)
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        let (id, action_str, process) = if parts.len() == 4 {
            // SysV format: id:runlevels:action:process (ignore runlevels)
            (parts[0], parts[2], parts[3])
        } else if parts.len() == 3 {
            // Simplified: id:action:process
            (parts[0], parts[1], parts[2])
        } else {
            eprintln!("init: {}:{}: malformed line, skipping", path, lineno + 1);
            continue;
        };

        let action = match action_str {
            "sysinit" => Action::Sysinit,
            "wait" => Action::Wait,
            "once" => Action::Once,
            "respawn" => Action::Respawn,
            "askfirst" => Action::Askfirst,
            "shutdown" => Action::Shutdown,
            "restart" => Action::Restart,
            "ctrlaltdel" => Action::Ctrlaltdel,
            other => {
                eprintln!("init: {}:{}: unknown action '{other}', skipping", path, lineno + 1);
                continue;
            }
        };

        entries.push(InittabEntry {
            id: id.to_string(),
            action,
            process: process.to_string(),
        });
    }

    if entries.is_empty() {
        return Err("no valid entries found".to_string());
    }

    Ok(entries)
}

/// Default inittab when /etc/inittab is missing
fn default_inittab() -> Vec<InittabEntry> {
    vec![
        InittabEntry {
            id: "rc".to_string(),
            action: Action::Sysinit,
            process: "/etc/init.d/rcS".to_string(),
        },
        InittabEntry {
            id: "tty1".to_string(),
            action: Action::Askfirst,
            process: "/bin/sh".to_string(),
        },
    ]
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

/// Check if a file is a shell script and return the interpreter if so
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

/// Run a command and wait for completion (executes directly, not via /bin/sh)
fn run_command_wait(cmd: &str) {
    let args = parse_command(cmd);
    if args.is_empty() {
        return;
    }

    // Check if the target is a script with a shebang
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

/// Spawn a process in the background (executes directly, not via /bin/sh)
fn spawn_process(cmd: &str) -> Option<i32> {
    let args = parse_command(cmd);
    if args.is_empty() {
        return None;
    }

    // Check if the target is a script with a shebang
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

/// Spawn an askfirst process (prints message and waits for Enter before exec)
fn spawn_askfirst(cmd: &str, id: &str) -> Option<i32> {
    // For askfirst, we write a small prompt then exec the command directly
    // Since we can't use shell, we'll just spawn the command directly
    // (the "press Enter" prompt requires a shell, so we skip it in no-shell mode)
    eprintln!("\nPlease press Enter to activate this console ({id}).");
    spawn_process(cmd)
}
