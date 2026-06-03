/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! modprobe — add or remove modules from the Linux kernel
//!
//! Usage: modprobe [-r] [-v] [-n] [-q] MODULE [params...]
//!
//! Loads a module and its dependencies using modules.dep.
//! With -r, removes a module (and unused dependencies).

use std::fs;
use std::io::Read;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    let mut remove = false;
    let mut verbose = false;
    let mut dry_run = false;
    let mut quiet = false;
    let mut module_name: Option<String> = None;
    let mut params: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-r" | "--remove" => remove = true,
            "-v" | "--verbose" => verbose = true,
            "-n" | "--dry-run" | "--show" => dry_run = true,
            "-q" | "--quiet" => quiet = true,
            "-h" | "--help" => {
                println!("Usage: modprobe [-r] [-v] [-n] [-q] MODULE [params...]");
                return 0;
            }
            s if !s.starts_with('-') => {
                if module_name.is_none() {
                    module_name = Some(s.to_string());
                } else {
                    params.push(s.to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }

    let module = match module_name {
        Some(m) => m,
        None => {
            eprintln!("modprobe: missing module name");
            return 1;
        }
    };

    // Normalize module name: replace - with _
    let normalized = module.replace('-', "_");

    if remove {
        return remove_module(&normalized, verbose, dry_run, quiet);
    }

    // Check if already loaded
    if is_loaded(&normalized) {
        if verbose {
            eprintln!("modprobe: {normalized} already loaded");
        }
        return 0;
    }

    // Find the kernel version
    let kver = get_kernel_version();
    let mod_dir = format!("/lib/modules/{kver}");

    // Read modules.dep to find the module file and its dependencies
    let dep_path = format!("{mod_dir}/modules.dep");
    let deps = match read_dependencies(&dep_path, &normalized) {
        Some(d) => d,
        None => {
            // Try to find the module file directly
            if let Some(path) = find_module_file(&mod_dir, &normalized) {
                vec![path]
            } else {
                if !quiet {
                    eprintln!("modprobe: FATAL: Module {normalized} not found");
                }
                return 1;
            }
        }
    };

    // Load dependencies first, then the module itself
    let mut ret = 0;
    for dep in &deps {
        let full_path = if dep.starts_with('/') {
            dep.clone()
        } else {
            format!("{mod_dir}/{dep}")
        };

        // Extract module name from path for loaded check
        let dep_name = Path::new(dep)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(dep)
            .trim_end_matches(".ko")
            .replace('-', "_");

        if is_loaded(&dep_name) {
            continue;
        }

        if verbose {
            eprintln!("modprobe: loading {dep}");
        }

        if dry_run {
            println!("insmod {full_path}");
            continue;
        }

        let mod_params = if dep == deps.last().unwrap_or(&String::new()) {
            params.join(" ")
        } else {
            String::new()
        };

        if load_module(&full_path, &mod_params) != 0 {
            if !quiet {
                eprintln!("modprobe: failed to load {dep}");
            }
            ret = 1;
            break;
        }
    }

    ret
}

/// Remove a module (with -r)
fn remove_module(name: &str, verbose: bool, dry_run: bool, quiet: bool) -> i32 {
    if !is_loaded(name) {
        if !quiet {
            eprintln!("modprobe: FATAL: Module {name} is not currently loaded");
        }
        return 1;
    }

    if verbose {
        eprintln!("modprobe: removing {name}");
    }

    if dry_run {
        println!("rmmod {name}");
        return 0;
    }

    let c_name = std::ffi::CString::new(name).unwrap();
    let flags = libc::O_NONBLOCK as libc::c_uint;
    let result = unsafe {
        libc::syscall(libc::SYS_delete_module, c_name.as_ptr(), flags)
    };

    if result != 0 {
        let err = std::io::Error::last_os_error();
        if !quiet {
            eprintln!("modprobe: can't unload '{name}': {err}");
        }
        return 1;
    }

    0
}

/// Check if a module is currently loaded
fn is_loaded(name: &str) -> bool {
    if let Ok(content) = fs::read_to_string("/proc/modules") {
        let normalized = name.replace('-', "_");
        content.lines().any(|line| {
            line.split_whitespace()
                .next()
                .map_or(false, |m| m == normalized)
        })
    } else {
        false
    }
}

/// Get the running kernel version
fn get_kernel_version() -> String {
    unsafe {
        let mut utsname: libc::utsname = std::mem::zeroed();
        libc::uname(&mut utsname);
        let release = std::ffi::CStr::from_ptr(utsname.release.as_ptr());
        release.to_string_lossy().to_string()
    }
}

/// Read modules.dep and return ordered list of files to load for a module
fn read_dependencies(dep_path: &str, module: &str) -> Option<Vec<String>> {
    let content = fs::read_to_string(dep_path).ok()?;
    let normalized = module.replace('-', "_");

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (mod_file, deps_str) = line.split_once(':')?;
        let mod_file = mod_file.trim();

        // Check if this line is for our module
        let mod_name = Path::new(mod_file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .trim_end_matches(".ko")
            .replace('-', "_");

        if mod_name == normalized {
            let mut result: Vec<String> = Vec::new();
            // Dependencies come first
            let deps_str = deps_str.trim();
            if !deps_str.is_empty() {
                for dep in deps_str.split_whitespace() {
                    result.push(dep.to_string());
                }
            }
            // The module itself is last
            result.push(mod_file.to_string());
            return Some(result);
        }
    }

    None
}

/// Find a module file in the modules directory
fn find_module_file(mod_dir: &str, name: &str) -> Option<String> {
    let normalized = name.replace('-', "_");
    find_module_recursive(Path::new(mod_dir), mod_dir, &normalized)
}

fn find_module_recursive(dir: &Path, base: &str, name: &str) -> Option<String> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_module_recursive(&path, base, name) {
                return Some(found);
            }
        } else if let Some(fname) = path.file_name().and_then(|f| f.to_str()) {
            let stem = fname
                .trim_end_matches(".zst")
                .trim_end_matches(".xz")
                .trim_end_matches(".gz")
                .trim_end_matches(".ko")
                .replace('-', "_");
            if stem == name {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }
    None
}

/// Load a kernel module from a file
fn load_module(path: &str, params: &str) -> i32 {
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("modprobe: {path}: {e}");
            return 1;
        }
    };

    let mut data = Vec::new();
    if let Err(e) = file.read_to_end(&mut data) {
        eprintln!("modprobe: error reading {path}: {e}");
        return 1;
    }

    let c_params = std::ffi::CString::new(params).unwrap();
    let ret = unsafe {
        libc::syscall(
            libc::SYS_init_module,
            data.as_ptr(),
            data.len(),
            c_params.as_ptr(),
        )
    };

    if ret != 0 {
        let err = std::io::Error::last_os_error();
        eprintln!("modprobe: {path}: {err}");
        return 1;
    }

    0
}
