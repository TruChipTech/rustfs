/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut show_all = false;
    let mut full_fmt = false;

    for arg in args {
        if let Some(flags) = arg.strip_prefix('-') {
            for c in flags.chars() {
                match c {
                    'e' | 'A' | 'a' | 'x' => show_all = true,
                    'f' | 'u' => full_fmt = true,
                    _ => {}
                }
            }
        }
    }

    let my_uid = unsafe { libc::getuid() };

    let mut pids: Vec<u32> = Vec::new();
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() {
                pids.push(pid);
            }
        }
    }
    pids.sort_unstable();

    if full_fmt {
        println!("{:<10} {:>7} {:>7}  {:<5}  {:>8}  COMMAND", "USER", "PID", "PPID", "STAT", "TIME");
    } else {
        println!("{:>7} {:<8} {:>9} {:<5} COMMAND", "PID", "USER", "VSZ", "STAT");
    }

    for pid in &pids {
        let Some(info) = read_proc(*pid) else { continue };

        if !show_all && info.uid != my_uid {
            continue;
        }

        let user = resolve_user(info.uid);
        let ticks = (info.utime + info.stime) / 100;
        let time_str = format!("{}:{:02}", ticks / 60, ticks % 60);

        if full_fmt {
            println!("{:<10} {:>7} {:>7}  {:<5}  {:>8}  {}",
                &user[..user.len().min(10)], pid, info.ppid, info.state, time_str, info.cmd);
        } else {
            println!("{:>7} {:<8} {:>9} {:<5} {}",
                pid, &user[..user.len().min(8)], info.vmsize, info.state, info.cmd);
        }
    }

    0
}

struct ProcInfo {
    uid: u32,
    ppid: u32,
    state: String,
    vmsize: u64,
    utime: u64,
    stime: u64,
    cmd: String,
}

fn read_proc(pid: u32) -> Option<ProcInfo> {
    let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;

    let mut uid: u32 = 0;
    let mut ppid: u32 = 0;
    let mut state = "?".to_string();
    let mut vmsize: u64 = 0;
    let mut proc_name = String::new();

    for line in status.lines() {
        if let Some(v) = line.strip_prefix("Name:\t") {
            proc_name = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("State:\t") {
            state = v.chars().next().unwrap_or('?').to_string();
        } else if let Some(v) = line.strip_prefix("PPid:\t") {
            ppid = v.trim().parse().unwrap_or(0);
        } else if let Some(v) = line.strip_prefix("Uid:\t") {
            uid = v.split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(0);
        } else if let Some(v) = line.strip_prefix("VmSize:\t") {
            vmsize = v.split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(0);
        }
    }

    let (utime, stime) = read_cpu_times(pid);

    let cmdline = fs::read(format!("/proc/{pid}/cmdline")).unwrap_or_default();
    let cmd = if cmdline.is_empty() {
        format!("[{proc_name}]")
    } else {
        let s: Vec<u8> = cmdline.iter().map(|&b| if b == 0 { b' ' } else { b }).collect();
        String::from_utf8_lossy(s.trim_ascii_end()).to_string()
    };

    Some(ProcInfo { uid, ppid, state, vmsize, utime, stime, cmd })
}

fn read_cpu_times(pid: u32) -> (u64, u64) {
    let stat = match fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(s) => s,
        Err(_) => return (0, 0),
    };
    let after_paren = stat.rfind(')').map(|i| i + 2).unwrap_or(stat.len());
    let fields: Vec<&str> = stat[after_paren..].split_whitespace().collect();
    let utime: u64 = fields.get(11).and_then(|s| s.parse().ok()).unwrap_or(0);
    let stime: u64 = fields.get(12).and_then(|s| s.parse().ok()).unwrap_or(0);
    (utime, stime)
}

fn resolve_user(uid: u32) -> String {
    if let Ok(content) = fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let mut parts = line.splitn(4, ':');
            let name = parts.next().unwrap_or("");
            let _ = parts.next(); // password
            let uid_str = parts.next().unwrap_or("");
            if uid_str.parse::<u32>().ok() == Some(uid) {
                return name.to_string();
            }
        }
    }
    uid.to_string()
}
