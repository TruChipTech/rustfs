/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! dnsdomainname — show the DNS domain part of the system's FQDN

pub fn run(_args: &[String]) -> i32 {
    let fqdn = match resolve_fqdn() {
        Some(f) => f,
        None => return 0,
    };
    if let Some((_, domain)) = fqdn.split_once('.') {
        println!("{domain}");
    } else {
        println!();
    }
    0
}

fn resolve_fqdn() -> Option<String> {
    let host = nix::unistd::gethostname().ok()?;
    let host = host.to_string_lossy().to_string();
    // If the hostname is already a FQDN, use it directly.
    if host.contains('.') {
        return Some(host);
    }
    // Otherwise fall back to /etc/resolv.conf's "domain"/"search" directive.
    if let Ok(content) = std::fs::read_to_string("/etc/resolv.conf") {
        for line in content.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("domain ").or_else(|| line.strip_prefix("search ")) {
                if let Some(d) = rest.split_whitespace().next() {
                    return Some(format!("{host}.{d}"));
                }
            }
        }
    }
    Some(host)
}
