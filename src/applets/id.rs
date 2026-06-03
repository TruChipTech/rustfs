/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
pub fn run(args: &[String]) -> i32 {
    let _show_user = args.is_empty() || args.iter().any(|a| a == "-u" || a == "--user");
    let _show_group = args.is_empty() || args.iter().any(|a| a == "-g" || a == "--group");
    let show_name = args.iter().any(|a| a == "-n" || a == "--name");

    #[cfg(unix)]
    {
        unsafe {
            let uid = libc::getuid();
            let gid = libc::getgid();
            let euid = libc::geteuid();
            let egid = libc::getegid();

            if args.iter().any(|a| a == "-u") {
                if show_name {
                    let pw = libc::getpwuid(uid);
                    if !pw.is_null() {
                        let name = std::ffi::CStr::from_ptr((*pw).pw_name)
                            .to_string_lossy();
                        println!("{name}");
                    }
                } else {
                    println!("{uid}");
                }
                return 0;
            }

            if args.iter().any(|a| a == "-g") {
                if show_name {
                    let gr = libc::getgrgid(gid);
                    if !gr.is_null() {
                        let name = std::ffi::CStr::from_ptr((*gr).gr_name)
                            .to_string_lossy();
                        println!("{name}");
                    }
                } else {
                    println!("{gid}");
                }
                return 0;
            }

            // Default: show all
            let mut parts = Vec::new();

            let pw = libc::getpwuid(uid);
            let uname = if !pw.is_null() {
                std::ffi::CStr::from_ptr((*pw).pw_name)
                    .to_string_lossy()
                    .to_string()
            } else {
                String::new()
            };

            let gr = libc::getgrgid(gid);
            let gname = if !gr.is_null() {
                std::ffi::CStr::from_ptr((*gr).gr_name)
                    .to_string_lossy()
                    .to_string()
            } else {
                String::new()
            };

            parts.push(format!("uid={uid}({uname})"));
            parts.push(format!("gid={gid}({gname})"));

            if euid != uid {
                parts.push(format!("euid={euid}"));
            }
            if egid != gid {
                parts.push(format!("egid={egid}"));
            }

            println!("{}", parts.join(" "));
        }
        0
    }

    #[cfg(not(unix))]
    {
        let username = std::env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string());
        println!("uid=1000({username}) gid=1000({username})");
        let _ = (show_user, show_group, show_name);
        0
    }
}
