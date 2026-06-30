/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! pivot_root — change the root filesystem
use std::ffi::CString;

pub fn run(args: &[String]) -> i32 {
    let positional: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if positional.len() != 2 {
        eprintln!("Usage: pivot_root NEW_ROOT PUT_OLD");
        return 1;
    }

    let new_root = match CString::new(positional[0].as_str()) { Ok(c) => c, Err(_) => return 1 };
    let put_old = match CString::new(positional[1].as_str()) { Ok(c) => c, Err(_) => return 1 };

    let ret = unsafe {
        libc::syscall(libc::SYS_pivot_root, new_root.as_ptr(), put_old.as_ptr())
    };
    if ret != 0 {
        eprintln!("pivot_root: {}", std::io::Error::last_os_error());
        return 1;
    }
    0
}
