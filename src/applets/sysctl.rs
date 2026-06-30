/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! sysctl — configure kernel parameters at runtime (via /proc/sys)
use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    let mut all = false;
    let mut write = false;
    let mut quiet = false;
    let mut from_file: Option<String> = None;
    let mut items: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "-A" => all = true,
            "-w" => write = true,
            "-q" => quiet = true,
            "-p" => { i += 1; from_file = Some(args.get(i).cloned().unwrap_or_else(|| "/etc/sysctl.conf".into())); }
            s => items.push(s.to_string()),
        }
        i += 1;
    }

    if all {
        return print_all();
    }
    if let Some(f) = from_file {
        return load_file(&f, quiet);
    }

    let mut rc = 0;
    for item in &items {
        if write || item.contains('=') {
            if let Some((k, v)) = item.split_once('=') {
                rc |= set_one(k.trim(), v.trim(), quiet);
            } else {
                eprintln!("sysctl: malformed setting: {item}");
                rc = 1;
            }
        } else {
            rc |= get_one(item, quiet);
        }
    }
    rc
}

fn key_to_path(key: &str) -> String {
    format!("/proc/sys/{}", key.replace('.', "/"))
}

fn path_to_key(path: &Path) -> String {
    path.strip_prefix("/proc/sys/")
        .unwrap_or(path)
        .to_string_lossy()
        .replace('/', ".")
}

fn get_one(key: &str, quiet: bool) -> i32 {
    let path = key_to_path(key);
    match fs::read_to_string(&path) {
        Ok(v) => {
            if quiet { println!("{}", v.trim_end()); }
            else { println!("{} = {}", key, v.trim_end().replace('\n', " ")); }
            0
        }
        Err(e) => { eprintln!("sysctl: {key}: {e}"); 1 }
    }
}

fn set_one(key: &str, val: &str, quiet: bool) -> i32 {
    let path = key_to_path(key);
    match fs::write(&path, val) {
        Ok(_) => {
            if !quiet { println!("{key} = {val}"); }
            0
        }
        Err(e) => { eprintln!("sysctl: {key}: {e}"); 1 }
    }
}

fn print_all() -> i32 {
    fn walk(dir: &Path) {
        if let Ok(entries) = fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p);
                } else if let Ok(v) = fs::read_to_string(&p) {
                    let v = v.trim_end();
                    if !v.contains('\n') {
                        println!("{} = {}", path_to_key(&p), v);
                    }
                }
            }
        }
    }
    walk(Path::new("/proc/sys"));
    0
}

fn load_file(file: &str, quiet: bool) -> i32 {
    let content = match fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => { eprintln!("sysctl: {file}: {e}"); return 1; }
    };
    let mut rc = 0;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') { continue; }
        if let Some((k, v)) = line.split_once('=') {
            rc |= set_one(k.trim(), v.trim(), quiet);
        }
    }
    rc
}
