/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        // Print all environment variables
        for (key, value) in std::env::vars() {
            println!("{key}={value}");
        }
    } else {
        for var in args {
            match std::env::var(var) {
                Ok(value) => println!("{value}"),
                Err(_) => return 1,
            }
        }
    }
    0
}
