/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! mkpasswd — compute a crypt(3) password hash.

use std::ffi::{CStr, CString};
use std::io::{self, Read, Write};

extern "C" {
    fn crypt(key: *const libc::c_char, salt: *const libc::c_char) -> *mut libc::c_char;
}

pub fn run(args: &[String]) -> i32 {
    let mut method = "sha512".to_string();
    let mut salt: Option<String> = None;
    let mut password: Option<String> = None;
    let mut positional: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-m" | "--method" => {
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
                eprintln!("Usage: mkpasswd [-m METHOD] [-S SALT] [PASSWORD [SALT]]");
                eprintln!("METHOD: des md5 sha256 sha512 (default sha512)");
                return 0;
            }
            s if s.starts_with('-') && s.len() > 1 => {
                eprintln!("mkpasswd: unknown option '{s}'");
                return 1;
            }
            _ => positional.push(args[i].clone()),
        }
        i += 1;
    }

    if !positional.is_empty() {
        password = Some(positional[0].clone());
    }
    if positional.len() > 1 && salt.is_none() {
        salt = Some(positional[1].clone());
    }

    let password = match password {
        Some(p) => p,
        None => {
            // Read passphrase from stdin (no echo control here; keep it simple).
            let mut s = String::new();
            if io::stdin().read_to_string(&mut s).is_err() {
                eprintln!("mkpasswd: cannot read password");
                return 1;
            }
            s.trim_end_matches(['\n', '\r']).to_string()
        }
    };

    let salt_str = build_salt(&method, salt.as_deref());
    match do_crypt(&password, &salt_str) {
        Some(h) => {
            let _ = writeln!(io::stdout(), "{h}");
            0
        }
        None => {
            eprintln!("mkpasswd: crypt failed (method '{method}' may be unsupported)");
            1
        }
    }
}

pub fn do_crypt(password: &str, salt: &str) -> Option<String> {
    let key = CString::new(password).ok()?;
    let salt_c = CString::new(salt).ok()?;
    unsafe {
        let res = crypt(key.as_ptr(), salt_c.as_ptr());
        if res.is_null() {
            return None;
        }
        let hashed = CStr::from_ptr(res).to_string_lossy().into_owned();
        if hashed.len() <= salt.len() && hashed == *salt {
            return None;
        }
        Some(hashed)
    }
}

pub fn build_salt(method: &str, given: Option<&str>) -> String {
    let prefix = match method {
        "des" => "",
        "md5" => "$1$",
        "sha256" => "$5$",
        _ => "$6$", // sha512 default
    };
    let rand = given.map(|s| s.to_string()).unwrap_or_else(|| random_salt(16));
    if prefix.is_empty() {
        // DES uses a 2-char salt.
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
    } else {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = ((t >> (i % 16)) as u8) ^ (i as u8);
        }
    }
    bytes
        .iter()
        .map(|b| ALPHABET[(*b as usize) % ALPHABET.len()] as char)
        .collect()
}
