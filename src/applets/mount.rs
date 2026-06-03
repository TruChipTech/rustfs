//! mount — mount a filesystem
//!
//! Usage: mount [-t type] [-o options] [-r] [-w] [device] [mountpoint]
//!        mount -a [-t type]       (mount all from /etc/fstab)
//!        mount                    (show mounted filesystems)

use std::ffi::CString;
use std::fs;

pub fn run(args: &[String]) -> i32 {
    let mut fstype: Option<String> = None;
    let mut options: Option<String> = None;
    let mut mount_all = false;
    let mut read_only = false;
    let mut remount = false;
    let mut bind = false;
    let mut positional: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-t" => {
                if let Some(t) = args.get(i + 1) {
                    fstype = Some(t.clone());
                    i += 2;
                    continue;
                } else {
                    eprintln!("mount: option -t requires an argument");
                    return 1;
                }
            }
            "-o" => {
                if let Some(o) = args.get(i + 1) {
                    options = Some(o.clone());
                    i += 2;
                    continue;
                } else {
                    eprintln!("mount: option -o requires an argument");
                    return 1;
                }
            }
            "-a" => mount_all = true,
            "-r" | "--read-only" => read_only = true,
            "-w" | "--rw" => read_only = false,
            "--bind" => bind = true,
            "-v" => {} // verbose (ignored)
            "-n" => {} // no mtab (ignored, we don't use mtab)
            "-h" | "--help" => {
                println!("Usage: mount [-t type] [-o options] [-r] [-w] [device] [mountpoint]");
                println!("       mount -a [-t type]");
                println!("       mount");
                return 0;
            }
            _ => {
                positional.push(args[i].clone());
            }
        }
        i += 1;
    }

    // Parse -o options for flags
    if let Some(ref opts) = options {
        for opt in opts.split(',') {
            match opt.trim() {
                "ro" => read_only = true,
                "rw" => read_only = false,
                "remount" => remount = true,
                "bind" => bind = true,
                _ => {}
            }
        }
    }

    // No args: show current mounts
    if !mount_all && positional.is_empty() {
        return show_mounts();
    }

    // -a: mount all from /etc/fstab
    if mount_all {
        return mount_fstab(fstype.as_deref());
    }

    // Need at least a mountpoint
    if positional.is_empty() {
        eprintln!("mount: missing operand");
        return 1;
    }

    let (device, mountpoint) = if positional.len() >= 2 {
        (positional[0].as_str(), positional[1].as_str())
    } else {
        // Single arg: try to find it in fstab
        let target = &positional[0];
        if let Some((dev, mp, ft, opts)) = find_in_fstab(target) {
            if fstype.is_none() {
                fstype = Some(ft);
            }
            if options.is_none() && !opts.is_empty() && opts != "defaults" {
                options = Some(opts);
            }
            // Leak is safe here — short-lived process
            let dev_leaked: &'static str = Box::leak(dev.into_boxed_str());
            let mp_leaked: &'static str = Box::leak(mp.into_boxed_str());
            (dev_leaked, mp_leaked)
        } else {
            eprintln!("mount: can't find {target} in /etc/fstab");
            return 1;
        }
    };

    do_mount(
        device,
        mountpoint,
        fstype.as_deref(),
        options.as_deref(),
        read_only,
        remount,
        bind,
    )
}

/// Show currently mounted filesystems from /proc/mounts
fn show_mounts() -> i32 {
    match fs::read_to_string("/proc/mounts") {
        Ok(content) => {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    println!("{} on {} type {} ({})", parts[0], parts[1], parts[2], parts[3]);
                }
            }
            0
        }
        Err(e) => {
            eprintln!("mount: /proc/mounts: {e}");
            1
        }
    }
}

/// Mount all entries from /etc/fstab
fn mount_fstab(filter_type: Option<&str>) -> i32 {
    let content = match fs::read_to_string("/etc/fstab") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("mount: /etc/fstab: {e}");
            return 1;
        }
    };

    let mut ret = 0;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        let device = parts[0];
        let mountpoint = parts[1];
        let fstype = parts[2];
        let opts = parts[3];

        // Skip root, swap, and already-mounted
        if mountpoint == "/" || mountpoint == "none" || fstype == "swap" {
            continue;
        }

        if let Some(ft) = filter_type {
            if fstype != ft {
                continue;
            }
        }

        if is_mounted(mountpoint) {
            continue;
        }

        let opt_str = if opts != "defaults" { Some(opts) } else { None };
        let ro = opts.split(',').any(|o| o == "ro");

        if do_mount(device, mountpoint, Some(fstype), opt_str, ro, false, false) != 0 {
            ret = 1;
        }
    }
    ret
}

/// Find a device or mountpoint in /etc/fstab
fn find_in_fstab(target: &str) -> Option<(String, String, String, String)> {
    let content = fs::read_to_string("/etc/fstab").ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        if parts[0] == target || parts[1] == target {
            return Some((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
                parts[3].to_string(),
            ));
        }
    }
    None
}

fn is_mounted(path: &str) -> bool {
    if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
        mounts.lines().any(|line| {
            line.split_whitespace()
                .nth(1)
                .map_or(false, |mp| mp == path)
        })
    } else {
        false
    }
}

/// Parse mount option flags
fn parse_mount_flags(options: Option<&str>, read_only: bool, remount: bool, bind: bool) -> (libc::c_ulong, String) {
    let mut flags: libc::c_ulong = 0;
    let mut data_opts: Vec<&str> = Vec::new();

    if read_only {
        flags |= libc::MS_RDONLY;
    }
    if remount {
        flags |= libc::MS_REMOUNT;
    }
    if bind {
        flags |= libc::MS_BIND;
    }

    if let Some(opts) = options {
        for opt in opts.split(',') {
            match opt.trim() {
                "ro" => flags |= libc::MS_RDONLY,
                "rw" => flags &= !libc::MS_RDONLY,
                "nosuid" => flags |= libc::MS_NOSUID,
                "suid" => flags &= !libc::MS_NOSUID,
                "nodev" => flags |= libc::MS_NODEV,
                "dev" => flags &= !libc::MS_NODEV,
                "noexec" => flags |= libc::MS_NOEXEC,
                "exec" => flags &= !libc::MS_NOEXEC,
                "sync" => flags |= libc::MS_SYNCHRONOUS,
                "async" => flags &= !libc::MS_SYNCHRONOUS,
                "remount" => flags |= libc::MS_REMOUNT,
                "bind" => flags |= libc::MS_BIND,
                "noatime" => flags |= libc::MS_NOATIME,
                "nodiratime" => flags |= libc::MS_NODIRATIME,
                "relatime" => flags |= libc::MS_RELATIME,
                "strictatime" => flags |= libc::MS_STRICTATIME,
                "defaults" => {}
                other => data_opts.push(other),
            }
        }
    }

    (flags, data_opts.join(","))
}

/// Perform the actual mount syscall
fn do_mount(
    device: &str,
    mountpoint: &str,
    fstype: Option<&str>,
    options: Option<&str>,
    read_only: bool,
    remount: bool,
    bind: bool,
) -> i32 {
    // Ensure mountpoint exists
    let _ = fs::create_dir_all(mountpoint);

    let (flags, data) = parse_mount_flags(options, read_only, remount, bind);

    let c_source = CString::new(device).unwrap();
    let c_target = CString::new(mountpoint).unwrap();

    // If no fstype given, try to auto-detect from common virtual fs names
    let detected_type = fstype.map(String::from).or_else(|| {
        match device {
            "proc" => Some("proc".to_string()),
            "sysfs" => Some("sysfs".to_string()),
            "devtmpfs" => Some("devtmpfs".to_string()),
            "devpts" => Some("devpts".to_string()),
            "tmpfs" => Some("tmpfs".to_string()),
            "cgroup2" | "cgroup" => Some("cgroup2".to_string()),
            _ => None,
        }
    });

    let c_fstype = detected_type
        .as_deref()
        .map(|t| CString::new(t).unwrap());

    let fstype_ptr = c_fstype
        .as_ref()
        .map_or(std::ptr::null(), |c| c.as_ptr());

    // Need to keep CString alive
    let c_data = if data.is_empty() {
        None
    } else {
        Some(CString::new(data.as_str()).unwrap())
    };

    let data_ptr = c_data
        .as_ref()
        .map_or(std::ptr::null(), |c| c.as_ptr() as *const libc::c_void);

    let ret = unsafe {
        libc::mount(
            c_source.as_ptr(),
            c_target.as_ptr(),
            fstype_ptr,
            flags,
            data_ptr,
        )
    };

    if ret != 0 {
        let err = std::io::Error::last_os_error();
        eprintln!("mount: mounting {device} on {mountpoint}: {err}");
        return 1;
    }

    0
}
