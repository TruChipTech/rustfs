/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! insmod — insert a kernel module

use std::fs;
use std::io::Read;

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        eprintln!("Usage: insmod MODULE [params...]");
        return if args.is_empty() { 1 } else { 0 };
    }

    let module_path = &args[0];
    let params = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        String::new()
    };

    // Read the module file
    let mut file = match fs::File::open(module_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("insmod: cannot open '{module_path}': {e}");
            return 1;
        }
    };

    let mut module_data = Vec::new();
    if let Err(e) = file.read_to_end(&mut module_data) {
        eprintln!("insmod: error reading '{module_path}': {e}");
        return 1;
    }

    // Use init_module syscall
    let params_cstr = std::ffi::CString::new(params.as_str()).unwrap();
    let ret = unsafe {
        libc::syscall(
            libc::SYS_init_module,
            module_data.as_ptr(),
            module_data.len(),
            params_cstr.as_ptr(),
        )
    };

    if ret != 0 {
        let err = std::io::Error::last_os_error();
        eprintln!("insmod: cannot insert '{module_path}': {err}");
        return 1;
    }

    0
}
