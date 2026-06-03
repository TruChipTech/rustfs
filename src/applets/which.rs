/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::env;
use std::path::Path;

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("which: missing operand");
        return 1;
    }

    let path_var = env::var("PATH").unwrap_or_default();
    let path_dirs: Vec<&str> = if cfg!(windows) {
        path_var.split(';').collect()
    } else {
        path_var.split(':').collect()
    };

    let mut exit_code = 0;

    for name in args {
        let mut found = false;

        // Check if it's an absolute path
        let path = Path::new(name);
        if path.is_absolute() && path.exists() {
            println!("{name}");
            found = true;
        } else {
            for dir in &path_dirs {
                let candidate = Path::new(dir).join(name);

                // On Windows, also try with common extensions
                #[cfg(windows)]
                {
                    let extensions = ["", ".exe", ".cmd", ".bat", ".com"];
                    for ext in &extensions {
                        let with_ext = if ext.is_empty() {
                            candidate.clone()
                        } else {
                            candidate.with_extension(&ext[1..])
                        };
                        if with_ext.exists() {
                            println!("{}", with_ext.display());
                            found = true;
                            break;
                        }
                    }
                    if found {
                        break;
                    }
                }

                #[cfg(not(windows))]
                {
                    if candidate.exists() {
                        println!("{}", candidate.display());
                        found = true;
                        break;
                    }
                }
            }
        }

        if !found {
            eprintln!("which: no {name} in ({})", path_var);
            exit_code = 1;
        }
    }

    exit_code
}
