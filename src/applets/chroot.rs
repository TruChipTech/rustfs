/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! chroot — run a command with a different root directory
//!
//! Usage: chroot NEWROOT [COMMAND [ARG]...]
//!
//! Changes the root directory to NEWROOT and then runs COMMAND (defaulting to
//! the shell `/bin/sh -i`). Must be run as root.

use std::ffi::CString;

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        eprintln!("Usage: chroot NEWROOT [COMMAND [ARG]...]");
        return if args.is_empty() { 1 } else { 0 };
    }

    let newroot = &args[0];

    // Default command is an interactive shell, like coreutils chroot.
    let (command, command_args): (String, Vec<String>) = if args.len() > 1 {
        (args[1].clone(), args[2..].to_vec())
    } else {
        (
            "/bin/sh".to_string(),
            vec!["-i".to_string()],
        )
    };

    let c_newroot = match CString::new(newroot.as_str()) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("chroot: invalid path '{newroot}'");
            return 1;
        }
    };

    if unsafe { libc::chroot(c_newroot.as_ptr()) } != 0 {
        eprintln!("chroot: cannot change root to '{newroot}': {}", std::io::Error::last_os_error());
        return 1;
    }

    // chroot does not change the working directory; move into the new root.
    let c_slash = CString::new("/").unwrap();
    if unsafe { libc::chdir(c_slash.as_ptr()) } != 0 {
        eprintln!("chroot: cannot chdir to /: {}", std::io::Error::last_os_error());
        return 1;
    }

    // Build argv for the command (argv[0] is the command itself).
    let c_prog = match CString::new(command.as_str()) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("chroot: invalid command '{command}'");
            return 1;
        }
    };
    let mut c_argv: Vec<CString> = vec![c_prog.clone()];
    for a in &command_args {
        match CString::new(a.as_str()) {
            Ok(c) => c_argv.push(c),
            Err(_) => {
                eprintln!("chroot: invalid argument");
                return 1;
            }
        }
    }
    let mut argv_ptrs: Vec<*const libc::c_char> = c_argv.iter().map(|c| c.as_ptr()).collect();
    argv_ptrs.push(std::ptr::null());

    // Replace this process with the command, searching PATH inside the new root.
    unsafe { libc::execvp(c_prog.as_ptr(), argv_ptrs.as_ptr()) };

    // execvp only returns on error.
    let err = std::io::Error::last_os_error();
    eprintln!("chroot: cannot run '{command}': {err}");
    if err.raw_os_error() == Some(libc::ENOENT) {
        127
    } else {
        126
    }
}
