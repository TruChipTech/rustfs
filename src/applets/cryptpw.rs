/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! cryptpw — crypt(3) a password read from stdin (or given on the command line).

use std::ffi::{CStr, CString};
use std::io::{self, Read, Write};

extern "C" {
    fn crypt(key: *const libc::c_char, salt: *const libc::c_char) -> *mut libc::c_char;
}

pub fn run(args: &[String]) -> i32 {
    let mut method = "sha512".to_string();
    let mut salt: Option<String> = None;
    let mut password: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-m" | "-P" | "--method" => {
                if i + 1 < args.len() {
                    method = args[i + 1].clone();
                    i += 1;
                }
            }
            "-S" | "--salt" => {
                if i + 1 < args.len() {
                    salt = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--help" => {
                eprintln!("Usage: cryptpw [-m METHOD] [-S SALT] [PASSWORD]");
                return 0;
            }
            s if s.starts_with('-') && s.len() > 1 => {
                eprintln!("cryptpw: unknown option '{s}'");
                return 1;
            }
            _ => password = Some(args[i].clone()),
        }
        i += 1;
    }

    let password = match password {
        Some(p) => p,
        None => {
            let mut s = String::new();
            if io::stdin().read_to_string(&mut s).is_err() {
                eprintln!("cryptpw: cannot read password");
                return 1;
            }
            s.trim_end_matches(['\n', '\r']).to_string()
        }
    };

    let salt_str = build_salt(&method, salt.as_deref());
    let key = match CString::new(password) {
        Ok(k) => k,
        Err(_) => return 1,
    };
    let salt_c = match CString::new(salt_str.clone()) {
        Ok(s) => s,
        Err(_) => return 1,
    };
    unsafe {
        let res = crypt(key.as_ptr(), salt_c.as_ptr());
        if res.is_null() {
            eprintln!("cryptpw: crypt failed");
            return 1;
        }
        let hashed = CStr::from_ptr(res).to_string_lossy().into_owned();
        let _ = writeln!(io::stdout(), "{hashed}");
    }
    0
}

fn build_salt(method: &str, given: Option<&str>) -> String {
    let prefix = match method {
        "des" => "",
        "md5" => "$1$",
        "sha256" => "$5$",
        _ => "$6$",
    };
    let rand = given.map(|s| s.to_string()).unwrap_or_else(|| random_salt(16));
    if prefix.is_empty() {
        rand.chars().take(2).collect()
    } else {
        format!("{prefix}{rand}")
    }
}

fn random_salt(n: usize) -> String {
    const ALPHABET: &[u8] = b"./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut bytes = vec![0u8; n];
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        let _ = f.read_exact(&mut bytes);
    }
    bytes
        .iter()
        .map(|b| ALPHABET[(*b as usize) % ALPHABET.len()] as char)
        .collect()
}
