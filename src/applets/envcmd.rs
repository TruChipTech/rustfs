/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::process::Command;

pub fn run(args: &[String]) -> i32 {
    let mut unset_vars = Vec::new();
    let mut set_vars = Vec::new();
    let mut command_start = None;
    let mut ignore_env = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-i" | "-" | "--ignore-environment" => ignore_env = true,
            "-u" | "--unset" => {
                i += 1;
                if i < args.len() {
                    unset_vars.push(args[i].clone());
                }
            }
            arg if arg.contains('=') => {
                set_vars.push(arg.to_string());
            }
            _ => {
                command_start = Some(i);
                break;
            }
        }
        i += 1;
    }

    if let Some(cmd_idx) = command_start {
        // Run command with modified environment
        let cmd = &args[cmd_idx];
        let cmd_args = &args[cmd_idx + 1..];

        let mut child = Command::new(cmd);
        child.args(cmd_args);

        if ignore_env {
            child.env_clear();
        }

        for var in &unset_vars {
            child.env_remove(var);
        }

        for var in &set_vars {
            if let Some(eq_pos) = var.find('=') {
                child.env(&var[..eq_pos], &var[eq_pos + 1..]);
            }
        }

        match child.status() {
            Ok(status) => status.code().unwrap_or(1),
            Err(e) => {
                eprintln!("env: {cmd}: {e}");
                127
            }
        }
    } else {
        // Print environment
        if ignore_env {
            for var in &set_vars {
                println!("{var}");
            }
        } else {
            for (key, value) in std::env::vars() {
                if !unset_vars.contains(&key) {
                    println!("{key}={value}");
                }
            }
            for var in &set_vars {
                println!("{var}");
            }
        }
        0
    }
}
