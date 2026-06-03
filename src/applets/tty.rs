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
        unsafe {
            if libc::isatty(0) == 1 {
                let result = libc::ttyname(0);
                if !result.is_null() {
                    let name = std::ffi::CStr::from_ptr(result)
                        .to_string_lossy();
                    println!("{name}");
                    return 0;
                }
            }
        }
        println!("not a tty");
        1
    }

    #[cfg(not(unix))]
    {
        // On Windows, check if stdin is a console
        println!("not a tty");
        1
    }
}
