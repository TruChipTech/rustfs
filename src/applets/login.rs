/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! login — begin a session on the system

use std::ffi::CString;
use std::io::{self, Write};

pub fn run(args: &[String]) -> i32 {
    let mut username = String::new();
    let mut preserve_env = false;

    for arg in args {
        match arg.as_str() {
            "-p" => preserve_env = true,
            "-h" | "--help" => {
                eprintln!("Usage: login [-p] [USERNAME]");
                return 0;
            }
            s if !s.starts_with('-') => username = s.to_string(),
            _ => {}
        }
    }

    // Get username if not provided
    if username.is_empty() {
        print!("login: ");
        let _ = io::stdout().flush();
        let mut buf = String::new();
        if io::stdin().read_line(&mut buf).is_err() {
            return 1;
        }
        username = buf.trim().to_string();
    }

    if username.is_empty() {
        return 1;
    }

    // Get password (disable echo)
    let password = read_password("Password: ");

    // Authenticate against /etc/shadow
    if !authenticate(&username, &password) {
        eprintln!("Login incorrect");
        return 1;
    }

    // Look up user info
    let (uid, gid, home, shell) = match get_user_info(&username) {
        Some(info) => info,
        None => {
            eprintln!("login: no such user");
            return 1;
        }
    };

    // Set up environment
    if !preserve_env {
        std::env::remove_var("MAIL");
    }
    std::env::set_var("HOME", &home);
    std::env::set_var("SHELL", &shell);
    std::env::set_var("USER", &username);
    std::env::set_var("LOGNAME", &username);
    std::env::set_var("PATH", "/bin:/sbin:/usr/bin:/usr/sbin");
    std::env::set_var("TERM", std::env::var("TERM").unwrap_or_else(|_| "linux".to_string()));

    // Change to home directory
    let _ = std::env::set_current_dir(&home);

    // Set uid/gid
    unsafe {
        libc::setgid(gid);
        libc::setuid(uid);
    }

    // Exec shell
    let c_shell = CString::new(shell.as_str()).unwrap();
    let shell_name = format!("-{}", std::path::Path::new(&shell)
        .file_name().and_then(|n| n.to_str()).unwrap_or("sh"));
    let c_arg0 = CString::new(shell_name.as_str()).unwrap();
    let args_arr = [c_arg0.as_ptr(), std::ptr::null()];

    unsafe { libc::execv(c_shell.as_ptr(), args_arr.as_ptr()) };
    eprintln!("login: exec {shell} failed");
    1
}

fn read_password(prompt: &str) -> String {
    eprint!("{prompt}");
    let _ = io::stderr().flush();

    // Disable echo
    let mut termios: libc::termios = unsafe { std::mem::zeroed() };
    unsafe { libc::tcgetattr(0, &mut termios) };
    let old_termios = termios;
    termios.c_lflag &= !libc::ECHO;
    unsafe { libc::tcsetattr(0, libc::TCSANOW, &termios) };

    let mut password = String::new();
    let _ = io::stdin().read_line(&mut password);

    // Restore echo
    unsafe { libc::tcsetattr(0, libc::TCSANOW, &old_termios) };
    eprintln!();

    password.trim().to_string()
}

fn authenticate(username: &str, password: &str) -> bool {
    // Read /etc/shadow
    let shadow = match std::fs::read_to_string("/etc/shadow") {
        Ok(c) => c,
        Err(_) => return false,
    };

    for line in shadow.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 2 && parts[0] == username {
            let hash = parts[1];
            if hash == "!" || hash == "*" || hash == "!!" {
                return false; // Locked account
            }
            if hash.is_empty() && password.is_empty() {
                return true; // No password set
            }
            // Verify using crypt
            let c_pass = CString::new(password).unwrap();
            let c_hash = CString::new(hash).unwrap();
            extern "C" { fn crypt(key: *const libc::c_char, salt: *const libc::c_char) -> *mut libc::c_char; }
            let result = unsafe { crypt(c_pass.as_ptr(), c_hash.as_ptr()) };
            if result.is_null() { return false; }
            let computed = unsafe { std::ffi::CStr::from_ptr(result) };
            return computed.to_bytes() == hash.as_bytes();
        }
    }
    false
}

fn get_user_info(username: &str) -> Option<(u32, u32, String, String)> {
    let passwd = std::fs::read_to_string("/etc/passwd").ok()?;
    for line in passwd.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 7 && parts[0] == username {
            let uid: u32 = parts[2].parse().ok()?;
            let gid: u32 = parts[3].parse().ok()?;
            let home = parts[5].to_string();
            let shell = parts[6].to_string();
            return Some((uid, gid, home, shell));
        }
    }
    None
}
