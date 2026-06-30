/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! findfs — find a filesystem device by label or UUID
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let spec = match args.iter().find(|a| !a.starts_with('-')) {
        Some(s) => s.clone(),
        None => { eprintln!("Usage: findfs LABEL=<label>|UUID=<uuid>"); return 1; }
    };

    let (dir, value) = if let Some(l) = spec.strip_prefix("LABEL=") {
        ("/dev/disk/by-label", l.to_string())
    } else if let Some(u) = spec.strip_prefix("UUID=") {
        ("/dev/disk/by-uuid", u.to_string())
    } else if let Some(p) = spec.strip_prefix("PARTUUID=") {
        ("/dev/disk/by-partuuid", p.to_string())
    } else if let Some(p) = spec.strip_prefix("PARTLABEL=") {
        ("/dev/disk/by-partlabel", p.to_string())
    } else {
        eprintln!("findfs: unsupported spec: {spec}");
        return 1;
    };

    let entry = format!("{dir}/{value}");
    match fs::canonicalize(&entry) {
        Ok(p) => { println!("{}", p.display()); 0 }
        Err(_) => {
            eprintln!("findfs: unable to resolve '{spec}'");
            1
        }
    }
}
