/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
pub fn run(_args: &[String]) -> i32 {
    #[cfg(unix)]
    {
        match std::env::var("USER") {
            Ok(user) => {
                println!("{user}");
                return 0;
            }
            Err(_) => {}
        }
        unsafe {
            let uid = libc::getuid();
            let pw = libc::getpwuid(uid);
            if !pw.is_null() {
                let name = std::ffi::CStr::from_ptr((*pw).pw_name)
                    .to_string_lossy();
                println!("{name}");
                return 0;
            }
        }
        eprintln!("whoami: cannot determine username");
        1
    }

    #[cfg(not(unix))]
    {
        match std::env::var("USERNAME") {
            Ok(user) => {
                println!("{user}");
                0
            }
            Err(_) => {
                eprintln!("whoami: cannot determine username");
                1
            }
        }
    }
}
