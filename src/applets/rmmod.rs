/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! rmmod — remove a kernel module
//!
//! Usage: rmmod [-f] [-w] MODULE...

pub fn run(args: &[String]) -> i32 {
    let mut force = false;
    let mut wait = false;
    let mut modules: Vec<String> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-f" | "--force" => force = true,
            "-w" | "--wait" => wait = true,
            "-h" | "--help" => {
                println!("Usage: rmmod [-f] [-w] MODULE...");
                return 0;
            }
            _ => modules.push(arg.clone()),
        }
    }

    if modules.is_empty() {
        eprintln!("rmmod: missing module name");
        return 1;
    }

    let mut ret = 0;
    for module in &modules {
        // Strip .ko/.ko.xz/.ko.gz suffix if present
        let name = module
            .rsplit('/')
            .next()
            .unwrap_or(module)
            .trim_end_matches(".ko.xz")
            .trim_end_matches(".ko.gz")
            .trim_end_matches(".ko")
            .replace('-', "_");

        let mut flags: libc::c_uint = libc::O_NONBLOCK as libc::c_uint;
        if force {
            flags |= libc::O_TRUNC as libc::c_uint;
        }
        if wait {
            flags &= !(libc::O_NONBLOCK as libc::c_uint);
        }

        let c_name = std::ffi::CString::new(name.as_str()).unwrap();
        let result = unsafe {
            libc::syscall(libc::SYS_delete_module, c_name.as_ptr(), flags)
        };

        if result != 0 {
            let err = std::io::Error::last_os_error();
            eprintln!("rmmod: can't unload '{name}': {err}");
            ret = 1;
        }
    }

    ret
}
