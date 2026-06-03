/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! ipcrm — remove IPC resources

pub fn run(args: &[String]) -> i32 {
    let mut i = 0;
    let mut exit_code = 0;

    if args.is_empty() {
        eprintln!("Usage: ipcrm [-m shmid] [-q msqid] [-s semid] [-M shmkey] [-Q msgkey] [-S semkey]");
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
            "-h" | "--help" => {
                eprintln!("Usage: ipcrm [-m shmid] [-q msqid] [-s semid] [-M shmkey] [-Q msgkey] [-S semkey]");
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
