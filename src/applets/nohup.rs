/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::fs::OpenOptions;
use std::os::unix::io::IntoRawFd;

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() || args[0] == "--help" {
        eprintln!("Usage: nohup COMMAND [ARG]...");
        if args.is_empty() { return 125; }
        return 0;
    }

    let cmd = &args[0];
    let cmd_args = &args[1..];

    // Ignore SIGHUP
    unsafe { libc::signal(libc::SIGHUP, libc::SIG_IGN); }

    // If stdout is a tty, redirect to nohup.out
    let stdout_is_tty = unsafe { libc::isatty(1) } == 1;
    if stdout_is_tty {
        if let Ok(f) = OpenOptions::new().create(true).append(true).open("nohup.out") {
            let fd = f.into_raw_fd();
            unsafe {
                libc::dup2(fd, 1);
                libc::close(fd);
            }
            eprintln!("nohup: appending output to 'nohup.out'");
        }
    }

    // Use execvp to replace the current process (in forked child from shell)
    let c_prog = match std::ffi::CString::new(cmd.as_str()) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("nohup: invalid command name");
            return 125;
        }
    };

    let c_args: Vec<std::ffi::CString> = std::iter::once(cmd.as_str())
        .chain(cmd_args.iter().map(|s| s.as_str()))
        .filter_map(|s| std::ffi::CString::new(s).ok())
        .collect();

    let c_arg_ptrs: Vec<*const libc::c_char> = c_args
        .iter()
        .map(|a| a.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect();

    unsafe { libc::execvp(c_prog.as_ptr(), c_arg_ptrs.as_ptr()); }

    // If execvp returns, it failed
    let err = std::io::Error::last_os_error();
    eprintln!("nohup: failed to run '{cmd}': {err}");
    127
}
