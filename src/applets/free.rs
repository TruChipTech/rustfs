/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::collections::HashMap;
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut human = false;
    let mut unit_div: u64 = 1; // keep kB (default)

    for arg in args {
        match arg.as_str() {
            "-h" | "--human-readable" => human = true,
            "-b" | "--bytes"          => unit_div = 0, // special: multiply kB×1024
            "-k" | "--kilo"           => unit_div = 1,
            "-m" | "--mega"           => unit_div = 1024,
            "-g" | "--giga"           => unit_div = 1024 * 1024,
            "--tebi" | "-T"           => unit_div = 1024 * 1024 * 1024,
            _ => {}
        }
    }

    let content = match fs::read_to_string("/proc/meminfo") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("free: /proc/meminfo: {e}");
            return 1;
        }
    };

    let mut info: HashMap<&str, u64> = HashMap::new();
    for line in content.lines() {
        if let Some((key, rest)) = line.split_once(':') {
            let val: u64 = rest.split_whitespace().next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            info.insert(key.trim(), val);
        }
    }

    // All values from /proc/meminfo are in kB
    let mem_total     = *info.get("MemTotal").unwrap_or(&0);
    let mem_free      = *info.get("MemFree").unwrap_or(&0);
    let mem_avail     = *info.get("MemAvailable").unwrap_or(&0);
    let shared        = *info.get("Shmem").unwrap_or(&0);
    let buffers       = *info.get("Buffers").unwrap_or(&0);
    let cached        = info.get("Cached").unwrap_or(&0)
                      + info.get("SReclaimable").unwrap_or(&0);
    let buff_cache    = buffers + cached;
    let mem_used      = mem_total.saturating_sub(mem_free + buff_cache);

    let swap_total    = *info.get("SwapTotal").unwrap_or(&0);
    let swap_free     = *info.get("SwapFree").unwrap_or(&0);
    let swap_used     = swap_total.saturating_sub(swap_free);

    let fmt = |kb: u64| -> String {
        if human {
            human_fmt(kb * 1024)
        } else if unit_div == 0 {
            format!("{}", kb * 1024)
        } else {
            format!("{}", kb / unit_div.max(1))
        }
    };

    println!("{:>15} {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
        "", "total", "used", "free", "shared", "buff/cache", "available");
    println!("{:<15} {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
        "Mem:", fmt(mem_total), fmt(mem_used), fmt(mem_free),
        fmt(shared), fmt(buff_cache), fmt(mem_avail));
    println!("{:<15} {:>12} {:>12} {:>12}",
        "Swap:", fmt(swap_total), fmt(swap_used), fmt(swap_free));

    0
}

fn human_fmt(bytes: u64) -> String {
    const UNITS: &[(&str, u64)] = &[
        ("Ti", 1u64 << 40),
        ("Gi", 1u64 << 30),
        ("Mi", 1u64 << 20),
        ("Ki", 1u64 << 10),
    ];
    for &(label, factor) in UNITS {
        if bytes >= factor {
            let v = bytes as f64 / factor as f64;
            return if v < 10.0 {
                format!("{:.1}{}", v, label)
            } else {
                format!("{:.0}{}", v, label)
            };
        }
    }
    format!("{}B", bytes)
}
