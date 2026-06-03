/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
use std::path::Path;

/// Implementation of the test / [ command
pub fn run(args: &[String]) -> i32 {
    // Strip trailing ] if invoked as [
    let args = if !args.is_empty() && args.last().map(|s| s.as_str()) == Some("]") {
        &args[..args.len() - 1]
    } else {
        args
    };

    if args.is_empty() {
        return 1; // false
    }

    match evaluate(args) {
        Ok(true) => 0,
        Ok(false) => 1,
        Err(e) => {
            eprintln!("test: {e}");
            2
        }
    }
}

fn evaluate(args: &[String]) -> Result<bool, String> {
    if args.len() == 1 {
        // Non-empty string is true
        return Ok(!args[0].is_empty());
    }

    if args.len() == 2 {
        match args[0].as_str() {
            "!" => return evaluate(&args[1..]).map(|v| !v),
            "-n" => return Ok(!args[1].is_empty()),
            "-z" => return Ok(args[1].is_empty()),
            "-f" => return Ok(Path::new(&args[1]).is_file()),
            "-d" => return Ok(Path::new(&args[1]).is_dir()),
            "-e" => return Ok(Path::new(&args[1]).exists()),
            "-s" => {
                return Ok(Path::new(&args[1])
                    .metadata()
                    .map(|m| m.len() > 0)
                    .unwrap_or(false));
            }
            "-r" => return Ok(Path::new(&args[1]).exists()), // simplified
            "-w" => {
                return Ok(!Path::new(&args[1])
                    .metadata()
                    .map(|m| m.permissions().readonly())
                    .unwrap_or(true));
            }
            "-x" => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    return Ok(Path::new(&args[1])
                        .metadata()
                        .map(|m| m.permissions().mode() & 0o111 != 0)
                        .unwrap_or(false));
                }
                #[cfg(not(unix))]
                {
                    return Ok(Path::new(&args[1]).exists());
                }
            }
            "-L" | "-h" => {
                return Ok(std::fs::symlink_metadata(&args[1])
                    .map(|m| m.file_type().is_symlink())
                    .unwrap_or(false));
            }
            _ => {}
        }
    }

    if args.len() == 3 {
        let op = &args[1];
        let left = &args[0];
        let right = &args[2];

        match op.as_str() {
            "=" | "==" => return Ok(left == right),
            "!=" => return Ok(left != right),
            "-eq" => {
                let l: i64 = left.parse().map_err(|_| format!("invalid integer: {left}"))?;
                let r: i64 = right.parse().map_err(|_| format!("invalid integer: {right}"))?;
                return Ok(l == r);
            }
            "-ne" => {
                let l: i64 = left.parse().map_err(|_| format!("invalid integer: {left}"))?;
                let r: i64 = right.parse().map_err(|_| format!("invalid integer: {right}"))?;
                return Ok(l != r);
            }
            "-lt" => {
                let l: i64 = left.parse().map_err(|_| format!("invalid integer: {left}"))?;
                let r: i64 = right.parse().map_err(|_| format!("invalid integer: {right}"))?;
                return Ok(l < r);
            }
            "-le" => {
                let l: i64 = left.parse().map_err(|_| format!("invalid integer: {left}"))?;
                let r: i64 = right.parse().map_err(|_| format!("invalid integer: {right}"))?;
                return Ok(l <= r);
            }
            "-gt" => {
                let l: i64 = left.parse().map_err(|_| format!("invalid integer: {left}"))?;
                let r: i64 = right.parse().map_err(|_| format!("invalid integer: {right}"))?;
                return Ok(l > r);
            }
            "-ge" => {
                let l: i64 = left.parse().map_err(|_| format!("invalid integer: {left}"))?;
                let r: i64 = right.parse().map_err(|_| format!("invalid integer: {right}"))?;
                return Ok(l >= r);
            }
            "-nt" => {
                // file1 is newer than file2
                let t1 = std::fs::metadata(left).and_then(|m| m.modified()).ok();
                let t2 = std::fs::metadata(right).and_then(|m| m.modified()).ok();
                return Ok(t1 > t2);
            }
            "-ot" => {
                let t1 = std::fs::metadata(left).and_then(|m| m.modified()).ok();
                let t2 = std::fs::metadata(right).and_then(|m| m.modified()).ok();
                return Ok(t1 < t2);
            }
            _ => {}
        }
    }

    // Handle -a (AND) and -o (OR)
    if let Some(pos) = args.iter().position(|a| a == "-o") {
        let left = evaluate(&args[..pos])?;
        let right = evaluate(&args[pos + 1..])?;
        return Ok(left || right);
    }

    if let Some(pos) = args.iter().position(|a| a == "-a") {
        let left = evaluate(&args[..pos])?;
        let right = evaluate(&args[pos + 1..])?;
        return Ok(left && right);
    }

    Err(format!("unknown expression: {}", args.join(" ")))
}
