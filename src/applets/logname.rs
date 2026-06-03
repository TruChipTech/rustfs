/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! logname — print user's login name

pub fn run(_args: &[String]) -> i32 {
    // Try getlogin first
    let login = unsafe { libc::getlogin() };
    if !login.is_null() {
        let name = unsafe { std::ffi::CStr::from_ptr(login) };
        if let Ok(s) = name.to_str() {
            println!("{s}");
            return 0;
        }
    }

    // Fallback to LOGNAME environment variable
    if let Ok(name) = std::env::var("LOGNAME") {
        println!("{name}");
        return 0;
    }

    eprintln!("logname: no login name");
    1
}
