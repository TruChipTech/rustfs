/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! ipcrm — remove IPC resources

use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut i = 0;
    let mut exit_code = 0;

    if args.is_empty() {
        eprintln!("Usage: ipcrm [-m shmid] [-q msqid] [-s semid] \
                   [-M shmkey] [-Q msgkey] [-S semkey] \
                   [--all=shm|msg|sem|all]");
        return 1;
    }

    while i < args.len() {
        match args[i].as_str() {
            "-m" => {
                i += 1;
                if i < args.len() {
                    let id: i32 = args[i].parse().unwrap_or(-1);
                    if unsafe { libc::shmctl(id, libc::IPC_RMID, std::ptr::null_mut()) } != 0 {
                        eprintln!("ipcrm: shmctl({}): {}", id, std::io::Error::last_os_error());
                        exit_code = 1;
                    }
                }
            }
            "-q" => {
                i += 1;
                if i < args.len() {
                    let id: i32 = args[i].parse().unwrap_or(-1);
                    if unsafe { libc::msgctl(id, libc::IPC_RMID, std::ptr::null_mut()) } != 0 {
                        eprintln!("ipcrm: msgctl({}): {}", id, std::io::Error::last_os_error());
                        exit_code = 1;
                    }
                }
            }
            "-s" => {
                i += 1;
                if i < args.len() {
                    let id: i32 = args[i].parse().unwrap_or(-1);
                    if unsafe { libc::semctl(id, 0, libc::IPC_RMID) } != 0 {
                        eprintln!("ipcrm: semctl({}): {}", id, std::io::Error::last_os_error());
                        exit_code = 1;
                    }
                }
            }
            "-M" => {
                i += 1;
                if i < args.len() {
                    let key: i32 = args[i].parse().unwrap_or(0);
                    let id = unsafe { libc::shmget(key, 0, 0) };
                    if id < 0 || unsafe { libc::shmctl(id, libc::IPC_RMID, std::ptr::null_mut()) } != 0 {
                        eprintln!("ipcrm: shmget/shmctl(key={}): {}", key, std::io::Error::last_os_error());
                        exit_code = 1;
                    }
                }
            }
            "-Q" => {
                i += 1;
                if i < args.len() {
                    let key: i32 = args[i].parse().unwrap_or(0);
                    let id = unsafe { libc::msgget(key, 0) };
                    if id < 0 || unsafe { libc::msgctl(id, libc::IPC_RMID, std::ptr::null_mut()) } != 0 {
                        eprintln!("ipcrm: msgget/msgctl(key={}): {}", key, std::io::Error::last_os_error());
                        exit_code = 1;
                    }
                }
            }
            "-S" => {
                i += 1;
                if i < args.len() {
                    let key: i32 = args[i].parse().unwrap_or(0);
                    let id = unsafe { libc::semget(key, 0, 0) };
                    if id < 0 || unsafe { libc::semctl(id, 0, libc::IPC_RMID) } != 0 {
                        eprintln!("ipcrm: semget/semctl(key={}): {}", key, std::io::Error::last_os_error());
                        exit_code = 1;
                    }
                }
            }
            // --all=TYPE  removes every resource of the given type.
            // TYPE may be: shm, msg, sem, or all (removes all three).
            // Column layout in /proc/sysvipc/*: key shmid/msqid/semid perms ...
            s if s.starts_with("--all=") || s == "--all" => {
                let type_str = s.strip_prefix("--all=").unwrap_or("all");
                let rm_shm = matches!(type_str, "shm" | "all");
                let rm_msg = matches!(type_str, "msg" | "all");
                let rm_sem = matches!(type_str, "sem" | "all");

                if !rm_shm && !rm_msg && !rm_sem {
                    eprintln!("ipcrm: unknown --all type: {type_str}; use shm, msg, sem, or all");
                    exit_code = 1;
                } else {
                    if rm_shm {
                        exit_code |= remove_all_from_proc("/proc/sysvipc/shm", |id| unsafe {
                            libc::shmctl(id, libc::IPC_RMID, std::ptr::null_mut())
                        }, "shmctl");
                    }
                    if rm_msg {
                        exit_code |= remove_all_from_proc("/proc/sysvipc/msg", |id| unsafe {
                            libc::msgctl(id, libc::IPC_RMID, std::ptr::null_mut())
                        }, "msgctl");
                    }
                    if rm_sem {
                        exit_code |= remove_all_from_proc("/proc/sysvipc/sem", |id| unsafe {
                            libc::semctl(id, 0, libc::IPC_RMID)
                        }, "semctl");
                    }
                }
            }
            "-h" | "--help" => {
                eprintln!("Usage: ipcrm [-m shmid] [-q msqid] [-s semid] \
                           [-M shmkey] [-Q msgkey] [-S semkey] \
                           [--all=shm|msg|sem|all]");
                return 0;
            }
            other => {
                eprintln!("ipcrm: unknown option: {other}");
                return 1;
            }
        }
        i += 1;
    }

    exit_code
}

/// Read IPC IDs from a /proc/sysvipc/* file (column 1 = id) and call
/// `remove_fn(id)` for each. Returns 1 if any removal failed.
fn remove_all_from_proc<F>(path: &str, mut remove_fn: F, syscall: &str) -> i32
where
    F: FnMut(i32) -> i32,
{
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ipcrm: cannot read {path}: {e}");
            return 1;
        }
    };

    let mut exit_code = 0;
    for line in content.lines().skip(1) {   // first line is the header
        let mut cols = line.split_whitespace();
        cols.next(); // skip key column
        if let Some(id_str) = cols.next() {
            if let Ok(id) = id_str.parse::<i32>() {
                if remove_fn(id) != 0 {
                    eprintln!("ipcrm: {}({}): {}", syscall, id, std::io::Error::last_os_error());
                    exit_code = 1;
                }
            }
        }
    }
    exit_code
}
