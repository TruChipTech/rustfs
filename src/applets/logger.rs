/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! logger — write messages to the system log

use std::ffi::CString;

pub fn run(args: &[String]) -> i32 {
    let mut priority = libc::LOG_USER | libc::LOG_NOTICE;
    let mut tag: Option<String> = None;
    let mut message_parts: Vec<String> = Vec::new();
    let mut stderr_too = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--priority" => {
                i += 1;
                if i < args.len() { priority = parse_priority(&args[i]); }
            }
            "-t" | "--tag" => {
                i += 1;
                if i < args.len() { tag = Some(args[i].clone()); }
            }
            "-s" | "--stderr" => stderr_too = true,
            "-h" | "--help" => {
                eprintln!("Usage: logger [-p PRIORITY] [-t TAG] [-s] MESSAGE...");
                return 0;
            }
            s => message_parts.push(s.to_string()),
        }
        i += 1;
    }

    let message = if message_parts.is_empty() {
        // Read from stdin
        let mut buf = String::new();
        if std::io::stdin().read_line(&mut buf).is_err() {
            return 1;
        }
        buf.trim().to_string()
    } else {
        message_parts.join(" ")
    };

    if stderr_too {
        eprintln!("{message}");
    }

    // Open syslog
    let ident = tag.unwrap_or_else(|| "logger".to_string());
    let c_ident = CString::new(ident.as_str()).unwrap();
    let c_msg = CString::new(message.as_str()).unwrap();

    unsafe {
        libc::openlog(c_ident.as_ptr(), libc::LOG_PID, libc::LOG_USER);
        libc::syslog(priority, c_msg.as_ptr());
        libc::closelog();
    }

    0
}

fn parse_priority(s: &str) -> i32 {
    // Format: facility.level or just level
    let (facility, level) = if let Some(dot) = s.find('.') {
        (&s[..dot], &s[dot + 1..])
    } else {
        ("user", s)
    };

    let fac = match facility {
        "kern" => libc::LOG_KERN,
        "user" => libc::LOG_USER,
        "mail" => libc::LOG_MAIL,
        "daemon" => libc::LOG_DAEMON,
        "auth" => libc::LOG_AUTH,
        "syslog" => libc::LOG_SYSLOG,
        "lpr" => libc::LOG_LPR,
        "news" => libc::LOG_NEWS,
        "uucp" => libc::LOG_UUCP,
        "cron" => libc::LOG_CRON,
        "local0" => libc::LOG_LOCAL0,
        "local1" => libc::LOG_LOCAL1,
        "local2" => libc::LOG_LOCAL2,
        "local3" => libc::LOG_LOCAL3,
        "local4" => libc::LOG_LOCAL4,
        "local5" => libc::LOG_LOCAL5,
        "local6" => libc::LOG_LOCAL6,
        "local7" => libc::LOG_LOCAL7,
        _ => libc::LOG_USER,
    };

    let lvl = match level {
        "emerg" => libc::LOG_EMERG,
        "alert" => libc::LOG_ALERT,
        "crit" => libc::LOG_CRIT,
        "err" | "error" => libc::LOG_ERR,
        "warning" | "warn" => libc::LOG_WARNING,
        "notice" => libc::LOG_NOTICE,
        "info" => libc::LOG_INFO,
        "debug" => libc::LOG_DEBUG,
        _ => libc::LOG_NOTICE,
    };

    fac | lvl
}
