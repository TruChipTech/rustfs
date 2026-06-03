/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
pub fn run(args: &[String]) -> i32 {
    let mut show_all = false;
    let mut show_sysname = false;
    let mut show_nodename = false;
    let mut show_release = false;
    let mut show_version = false;
    let mut show_machine = false;

    if args.is_empty() {
        show_sysname = true;
    }

    for arg in args {
        match arg.as_str() {
            "-a" | "--all" => show_all = true,
            "-s" | "--kernel-name" => show_sysname = true,
            "-n" | "--nodename" => show_nodename = true,
            "-r" | "--kernel-release" => show_release = true,
            "-v" | "--kernel-version" => show_version = true,
            "-m" | "--machine" => show_machine = true,
            _ => {}
        }
    }

    if show_all {
        show_sysname = true;
        show_nodename = true;
        show_release = true;
        show_version = true;
        show_machine = true;
    }

    let mut parts = Vec::new();

    #[cfg(unix)]
    {
        unsafe {
            let mut uts: libc::utsname = std::mem::zeroed();
            if libc::uname(&mut uts) == 0 {
                let sysname = std::ffi::CStr::from_ptr(uts.sysname.as_ptr())
                    .to_string_lossy()
                    .to_string();
                let nodename = std::ffi::CStr::from_ptr(uts.nodename.as_ptr())
                    .to_string_lossy()
                    .to_string();
                let release = std::ffi::CStr::from_ptr(uts.release.as_ptr())
                    .to_string_lossy()
                    .to_string();
                let version = std::ffi::CStr::from_ptr(uts.version.as_ptr())
                    .to_string_lossy()
                    .to_string();
                let machine = std::ffi::CStr::from_ptr(uts.machine.as_ptr())
                    .to_string_lossy()
                    .to_string();

                if show_sysname { parts.push(sysname); }
                if show_nodename { parts.push(nodename); }
                if show_release { parts.push(release); }
                if show_version { parts.push(version); }
                if show_machine { parts.push(machine); }
            }
        }
    }

    #[cfg(not(unix))]
    {
        if show_sysname {
            parts.push("Windows".to_string());
        }
        if show_nodename {
            parts.push(std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".to_string()));
        }
        if show_release {
            parts.push("10.0".to_string());
        }
        if show_version {
            parts.push("Windows NT".to_string());
        }
        if show_machine {
            parts.push(std::env::var("PROCESSOR_ARCHITECTURE").unwrap_or_else(|_| "x86_64".to_string()));
        }
    }

    if parts.is_empty() {
        parts.push("unknown".to_string());
    }

    println!("{}", parts.join(" "));
    0
}
