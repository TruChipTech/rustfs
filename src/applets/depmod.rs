//! depmod — generate modules.dep and map files
//!
//! Usage: depmod [-a] [-b basedir] [-n] [version]
//!
//! Scans /lib/modules/<version>/ for kernel modules and generates:
//!   modules.dep      — module dependency list
//!   modules.alias    — module alias mappings
//!   modules.symbols  — exported symbol mappings

use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    let mut basedir = String::new();
    let mut dry_run = false;
    let mut version: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "-A" => {} // scan all (default)
            "-b" | "--basedir" => {
                if let Some(b) = args.get(i + 1) {
                    basedir = b.clone();
                    i += 2;
                    continue;
                }
            }
            "-n" | "--dry-run" => dry_run = true,
            "-h" | "--help" => {
                println!("Usage: depmod [-a] [-b basedir] [-n] [version]");
                return 0;
            }
            s if !s.starts_with('-') => {
                version = Some(s.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    // Determine kernel version
    let kver = version.unwrap_or_else(|| {
        unsafe {
            let mut utsname: libc::utsname = std::mem::zeroed();
            libc::uname(&mut utsname);
            let release = std::ffi::CStr::from_ptr(utsname.release.as_ptr());
            release.to_string_lossy().to_string()
        }
    });

    let mod_dir = if basedir.is_empty() {
        format!("/lib/modules/{kver}")
    } else {
        format!("{basedir}/lib/modules/{kver}")
    };

    if !Path::new(&mod_dir).is_dir() {
        eprintln!("depmod: {mod_dir}: No such directory");
        return 1;
    }

    eprintln!("depmod: scanning {mod_dir}...");

    // Find all .ko, .ko.gz, .ko.xz files
    let mut modules: Vec<String> = Vec::new();
    find_modules(Path::new(&mod_dir), &mod_dir, &mut modules);

    eprintln!("depmod: found {} modules", modules.len());

    // Build dependency map (simplified — real depmod reads ELF symbols)
    // For each module, record its relative path
    let mut dep_map: HashMap<String, Vec<String>> = HashMap::new();
    for module in &modules {
        dep_map.insert(module.clone(), Vec::new());
    }

    // Generate modules.dep
    let mut dep_content = String::new();
    for module in &modules {
        let deps = dep_map.get(module).cloned().unwrap_or_default();
        if deps.is_empty() {
            dep_content.push_str(&format!("{module}:\n"));
        } else {
            dep_content.push_str(&format!("{}: {}\n", module, deps.join(" ")));
        }
    }

    // Generate modules.dep.bin (we just write the text version)
    // Generate modules.alias (empty for now — real depmod reads MODULE_ALIAS)
    let alias_content = String::from("# Aliases extracted from modules themselves.\n");

    // Generate modules.symbols (empty for now)
    let symbols_content = String::from("# Aliases for symbols, used by symbol_request().\n");

    if dry_run {
        print!("{dep_content}");
        return 0;
    }

    // Write files
    let dep_path = format!("{mod_dir}/modules.dep");
    let alias_path = format!("{mod_dir}/modules.alias");
    let symbols_path = format!("{mod_dir}/modules.symbols");

    if let Err(e) = fs::write(&dep_path, &dep_content) {
        eprintln!("depmod: {dep_path}: {e}");
        return 1;
    }
    let _ = fs::write(&alias_path, &alias_content);
    let _ = fs::write(&symbols_path, &symbols_content);

    eprintln!("depmod: wrote {dep_path}");
    0
}

/// Recursively find kernel module files
fn find_modules(dir: &Path, base: &str, modules: &mut Vec<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            find_modules(&path, base, modules);
        } else if let Some(name) = path.to_str() {
            if name.ends_with(".ko")
                || name.ends_with(".ko.gz")
                || name.ends_with(".ko.xz")
                || name.ends_with(".ko.zst")
            {
                // Store relative path from module dir
                if let Some(rel) = name.strip_prefix(base) {
                    let rel = rel.trim_start_matches('/');
                    modules.push(rel.to_string());
                }
            }
        }
    }
}
