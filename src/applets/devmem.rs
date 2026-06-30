/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! devmem — read or write physical memory via /dev/mem
use std::os::unix::io::AsRawFd;

pub fn run(args: &[String]) -> i32 {
    let pos: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if pos.is_empty() {
        eprintln!("Usage: devmem ADDRESS [WIDTH [VALUE]]");
        return 1;
    }
    let addr = match parse_num(pos[0]) { Some(a) => a, None => { eprintln!("devmem: bad address"); return 1; } };
    let width: usize = pos.get(1).and_then(|s| parse_num(s)).unwrap_or(32) as usize / 8;
    let width = if width == 0 { 4 } else { width };
    let value = pos.get(2).and_then(|s| parse_num(s));

    let file = match std::fs::OpenOptions::new().read(true).write(value.is_some()).open("/dev/mem") {
        Ok(f) => f,
        Err(e) => { eprintln!("devmem: /dev/mem: {e}"); return 1; }
    };

    let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64;
    let base = addr & !(page - 1);
    let off = (addr - base) as usize;

    let prot = if value.is_some() { libc::PROT_READ | libc::PROT_WRITE } else { libc::PROT_READ };
    let map = unsafe {
        libc::mmap(std::ptr::null_mut(), page as usize, prot, libc::MAP_SHARED,
            file.as_raw_fd(), base as libc::off_t)
    };
    if map == libc::MAP_FAILED {
        eprintln!("devmem: mmap: {}", std::io::Error::last_os_error());
        return 1;
    }

    let ptr = unsafe { (map as *mut u8).add(off) };
    let rc;
    unsafe {
        if let Some(v) = value {
            write_at(ptr, v, width);
            let read = read_at(ptr, width);
            println!("0x{read:0w$X}", w = width * 2);
        } else {
            let read = read_at(ptr, width);
            println!("0x{read:0w$X}", w = width * 2);
        }
        rc = 0;
        libc::munmap(map, page as usize);
    }
    rc
}

unsafe fn read_at(ptr: *const u8, width: usize) -> u64 {
    match width {
        1 => ptr.read_volatile() as u64,
        2 => (ptr as *const u16).read_volatile() as u64,
        8 => (ptr as *const u64).read_volatile(),
        _ => (ptr as *const u32).read_volatile() as u64,
    }
}

unsafe fn write_at(ptr: *mut u8, v: u64, width: usize) {
    match width {
        1 => ptr.write_volatile(v as u8),
        2 => (ptr as *mut u16).write_volatile(v as u16),
        8 => (ptr as *mut u64).write_volatile(v),
        _ => (ptr as *mut u32).write_volatile(v as u32),
    }
}

fn parse_num(s: &str) -> Option<u64> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(h, 16).ok()
    } else {
        s.parse().ok()
    }
}
