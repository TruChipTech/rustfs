/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! modinfo — show information about a kernel module
//!
//! Usage: modinfo [-F field] MODULE...
//!
//! Displays information from .modinfo section of kernel module files.

use std::fs;
use std::io::Read;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    let mut field_filter: Option<String> = None;
    let mut modules: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-F" | "--field" | "-f" => {
                if let Some(f) = args.get(i + 1) {
                    field_filter = Some(f.clone());
                    i += 2;
                    continue;
                } else {
                    eprintln!("modinfo: option -F requires an argument");
                    return 1;
                }
            }
            "-h" | "--help" => {
                println!("Usage: modinfo [-F field] MODULE...");
                return 0;
            }
            _ => modules.push(args[i].clone()),
        }
        i += 1;
    }

    if modules.is_empty() {
        eprintln!("modinfo: missing module name");
        return 1;
    }

    let kver = get_kernel_version();
    let mod_dir = format!("/lib/modules/{kver}");

    let mut ret = 0;
    for module in &modules {
        if show_module_info(module, &mod_dir, field_filter.as_deref()) != 0 {
            ret = 1;
        }
    }
    ret
}

fn show_module_info(module: &str, mod_dir: &str, field_filter: Option<&str>) -> i32 {
    // Find the module file
    let path = if Path::new(module).exists() {
        module.to_string()
    } else {
        let normalized = module
            .trim_end_matches(".ko.xz")
            .trim_end_matches(".ko.gz")
            .trim_end_matches(".ko")
            .replace('-', "_");
        match find_module_file(mod_dir, &normalized) {
            Some(p) => p,
            None => {
                eprintln!("modinfo: ERROR: Module {module} not found");
                return 1;
            }
        }
    };

    // Read the module file
    let data = match read_module_data(&path) {
        Some(d) => d,
        None => return 1,
    };

    // Extract .modinfo section strings
    let infos = extract_modinfo(&data);

    // Always show filename
    if let Some(filter) = field_filter {
        for (key, value) in &infos {
            if key == filter {
                println!("{value}");
            }
        }
        if filter == "filename" {
            println!("{path}");
        }
    } else {
        println!("filename:       {path}");
        for (key, value) in &infos {
            println!("{key:15} {value}");
        }
    }

    0
}

/// Read module data (handles .ko, .ko.gz, .ko.xz)
fn read_module_data(path: &str) -> Option<Vec<u8>> {
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("modinfo: {path}: {e}");
            return None;
        }
    };

    let mut data = Vec::new();
    if let Err(e) = file.read_to_end(&mut data) {
        eprintln!("modinfo: {path}: {e}");
        return None;
    }

    Some(data)
}

/// Extract key=value pairs from .modinfo section of an ELF module
fn extract_modinfo(data: &[u8]) -> Vec<(String, String)> {
    let mut results = Vec::new();

    // Simple approach: scan the binary for null-terminated strings that look like key=value
    // The .modinfo section contains null-terminated key=value strings
    // We look for common modinfo keys in the binary data

    let known_keys = [
        "license", "description", "author", "alias", "depends", "retpoline",
        "name", "vermagic", "srcversion", "intree", "parm", "parmtype",
        "firmware", "version", "sig_id", "signer", "sig_key",
    ];

    // Find all null-terminated strings that contain '='
    let mut i = 0;
    while i < data.len() {
        // Look for start of a potential modinfo string
        if data[i] == 0 || !data[i].is_ascii_alphabetic() {
            i += 1;
            continue;
        }

        // Find the end of this null-terminated string
        let start = i;
        while i < data.len() && data[i] != 0 {
            i += 1;
        }

        if i > start && i - start < 4096 {
            if let Ok(s) = std::str::from_utf8(&data[start..i]) {
                if let Some((key, value)) = s.split_once('=') {
                    // Check if this looks like a modinfo entry
                    if known_keys.contains(&key)
                        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    {
                        results.push((key.to_string(), value.to_string()));
                    }
                }
            }
        }
        i += 1;
    }

    results
}

/// Find a module file in the modules directory
fn find_module_file(mod_dir: &str, name: &str) -> Option<String> {
    find_module_recursive(Path::new(mod_dir), name)
}

fn find_module_recursive(dir: &Path, name: &str) -> Option<String> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_module_recursive(&path, name) {
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

fn get_kernel_version() -> String {
    unsafe {
        let mut utsname: libc::utsname = std::mem::zeroed();
        libc::uname(&mut utsname);
        let release = std::ffi::CStr::from_ptr(utsname.release.as_ptr());
        release.to_string_lossy().to_string()
    }
}
