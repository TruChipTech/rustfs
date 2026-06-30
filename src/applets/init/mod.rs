/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
#[cfg(init_rustfs)]
pub mod rustfs_init;

#[cfg(init_sysvinit)]
pub mod sysvinit;

#[cfg(init_systemd)]
pub mod systemd_compat;

/// Main init entry point — dispatches to the configured init system.
/// Exactly one init system is compiled in at a time (enforced by Kconfig choice).
#[allow(unreachable_code)]
pub fn run(args: &[String]) -> i32 {
    #[cfg(not(unix))]
    {
        let _ = args;
        eprintln!("init: only supported on Unix/Linux systems");
        return 1;
    }

    #[cfg(unix)]
    {
        #[cfg(init_rustfs)]
        {
            return rustfs_init::run(args);
        }

        #[cfg(init_sysvinit)]
        {
            return sysvinit::run(args);
        }

        #[cfg(init_systemd)]
        {
            systemd_compat::run(args)
        }

        #[cfg(not(any(init_rustfs, init_sysvinit, init_systemd)))]
        {
            let _ = args;
            eprintln!("init: no init system compiled in");
            eprintln!("Enable CONFIG_INIT_RUSTFS, CONFIG_INIT_SYSVINIT, or CONFIG_INIT_SYSTEMD in .config");
            1
        }
    }
}
