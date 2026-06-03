/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
pub fn run(args: &[String]) -> i32 {
    #[cfg(unix)]
    {
        // Parse arguments
        let mut new_hostname: Option<String> = None;
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-F" | "--file" => {
                    if let Some(path) = args.get(i + 1) {
                        match std::fs::read_to_string(path) {
                            Ok(content) => {
                                let name = content.trim().to_string();
                                if !name.is_empty() {
                                    new_hostname = Some(name);
                                }
                            }
                            Err(e) => {
                                eprintln!("hostname: {path}: {e}");
                                return 1;
                            }
                        }
                        i += 2;
                        continue;
                    } else {
                        eprintln!("hostname: option requires an argument -- 'F'");
                        return 1;
                    }
                }
                s if !s.starts_with('-') => {
                    new_hostname = Some(s.to_string());
                }
                _ => {}
            }
            i += 1;
        }

        // Set hostname if requested
        if let Some(ref name) = new_hostname {
            let c_name = std::ffi::CString::new(name.as_str()).unwrap();
            let ret = unsafe { libc::sethostname(c_name.as_ptr(), name.len()) };
            if ret != 0 {
                eprintln!("hostname: sethostname: {}", std::io::Error::last_os_error());
                return 1;
            }
            return 0;
        }

        // Print current hostname
        let mut buf = vec![0u8; 256];
        unsafe {
            if libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) == 0 {
                let name = std::ffi::CStr::from_ptr(buf.as_ptr() as *const libc::c_char)
                    .to_string_lossy();
                println!("{name}");
                return 0;
            }
        }
        eprintln!("hostname: failed to get hostname");
        1
    }

    #[cfg(not(unix))]
    {
        let _ = args;
        match std::env::var("COMPUTERNAME") {
            Ok(name) => {
                println!("{name}");
                0
            }
            Err(_) => {
                eprintln!("hostname: failed to get hostname");
                1
            }
        }
    }
}
