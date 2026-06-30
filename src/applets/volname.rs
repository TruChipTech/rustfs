/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! volname — print the volume label of an ISO-9660 filesystem (e.g. a CD-ROM).

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

// The ISO-9660 Primary Volume Descriptor lives at sector 16 (2048 bytes each);
// the 32-byte volume identifier starts at offset 40 within it.
const PVD_VOLID_OFFSET: u64 = 16 * 2048 + 40;

pub fn run(args: &[String]) -> i32 {
    let dev = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str())
        .unwrap_or("/dev/cdrom");

    let mut f = match File::open(dev) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("volname: {dev}: {e}");
            return 1;
        }
    };
    if f.seek(SeekFrom::Start(PVD_VOLID_OFFSET)).is_err() {
        eprintln!("volname: {dev}: cannot seek");
        return 1;
    }
    let mut buf = [0u8; 32];
    if f.read_exact(&mut buf).is_err() {
        eprintln!("volname: {dev}: cannot read volume descriptor");
        return 1;
    }
    let label = String::from_utf8_lossy(&buf);
    println!("{}", label.trim_end());
    0
}
