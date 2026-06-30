/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! mkswap — set up a Linux swap area (v1 header)
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::io::AsRawFd;

const BLKGETSIZE64: libc::c_ulong = 0x80081272;

pub fn run(args: &[String]) -> i32 {
    let pos: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if pos.is_empty() {
        eprintln!("Usage: mkswap DEVICE [SIZE_KiB]");
        return 1;
    }
    let dev = pos[0].clone();

    let mut file = match OpenOptions::new().read(true).write(true).open(&dev) {
        Ok(f) => f,
        Err(e) => { eprintln!("mkswap: {dev}: {e}"); return 1; }
    };

    let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64;

    // Determine size in bytes: explicit KiB arg, block device size, or file len.
    let size_bytes = if let Some(kib) = pos.get(1).and_then(|s| s.parse::<u64>().ok()) {
        kib * 1024
    } else {
        let mut sz: u64 = 0;
        if unsafe { libc::ioctl(file.as_raw_fd(), BLKGETSIZE64 as _, &mut sz) } == 0 && sz > 0 {
            sz
        } else {
            file.metadata().map(|m| m.len()).unwrap_or(0)
        }
    };

    if size_bytes < page * 10 {
        eprintln!("mkswap: swap area too small");
        return 1;
    }
    let last_page = (size_bytes / page) - 1;

    // swap_header_v1_2: layout starts at offset 1024.
    // u32 version=1, u32 last_page, u32 nr_badpages, 16 uuid, 16 volume label
    let mut header = vec![0u8; page as usize];
    let off = 1024usize;
    header[off..off + 4].copy_from_slice(&1u32.to_le_bytes());
    header[off + 4..off + 8].copy_from_slice(&(last_page as u32).to_le_bytes());
    // nr_badpages = 0 (already zeroed)
    // Magic "SWAPSPACE2" at the very end of the first page.
    let magic = b"SWAPSPACE2";
    let mstart = page as usize - magic.len();
    header[mstart..].copy_from_slice(magic);

    if let Err(e) = file.seek(SeekFrom::Start(0)).and_then(|_| file.write_all(&header)) {
        eprintln!("mkswap: {dev}: {e}");
        return 1;
    }
    let _ = file.flush();
    println!("Setting up swapspace version 1, size = {} KiB", size_bytes / 1024);
    0
}
