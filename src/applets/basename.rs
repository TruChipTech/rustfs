/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("basename: missing operand");
        return 1;
    }

    let name = &args[0];
    let suffix = args.get(1).map(|s| s.as_str());

    let base = Path::new(name)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let result = if let Some(suffix) = suffix {
        if base.ends_with(suffix) && base.len() > suffix.len() {
            base[..base.len() - suffix.len()].to_string()
        } else {
            base
        }
    } else {
        base
    };

    println!("{result}");
    0
}
