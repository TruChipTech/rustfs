/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
/// Implementation of the expr command
pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("expr: missing operand");
        return 2;
    }

    match evaluate(args) {
        Ok(value) => {
            println!("{value}");
            // expr returns 1 if the result is null/zero
            if value == "0" || value.is_empty() {
                1
            } else {
                0
            }
        }
        Err(e) => {
            eprintln!("expr: {e}");
            2
        }
    }
}

fn evaluate(args: &[String]) -> Result<String, String> {
    if args.len() == 1 {
        return Ok(args[0].clone());
    }

    // Handle OR (|)
    if let Some(pos) = args.iter().rposition(|a| a == "|") {
        let left = evaluate(&args[..pos])?;
        if !left.is_empty() && left != "0" {
            return Ok(left);
        }
        return evaluate(&args[pos + 1..]);
    }

    // Handle AND (&)
    if let Some(pos) = args.iter().rposition(|a| a == "&") {
        let left = evaluate(&args[..pos])?;
        let right = evaluate(&args[pos + 1..])?;
        if (!left.is_empty() && left != "0") && (!right.is_empty() && right != "0") {
            return Ok(left);
        }
        return Ok("0".to_string());
    }

    // Handle comparison operators
    for op in &["<", "<=", "=", "!=", ">=", ">"] {
        if let Some(pos) = args.iter().rposition(|a| a == *op) {
            let left = evaluate(&args[..pos])?;
            let right = evaluate(&args[pos + 1..])?;

            // Try numeric comparison first
            let result = if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
                match *op {
                    "<" => l < r,
                    "<=" => l <= r,
                    "=" => l == r,
                    "!=" => l != r,
                    ">=" => l >= r,
                    ">" => l > r,
                    _ => false,
                }
            } else {
                match *op {
                    "<" => left < right,
                    "<=" => left <= right,
                    "=" => left == right,
                    "!=" => left != right,
                    ">=" => left >= right,
                    ">" => left > right,
                    _ => false,
                }
            };

            return Ok(if result { "1" } else { "0" }.to_string());
        }
    }

    // Handle + and -
    if let Some(pos) = args.iter().rposition(|a| a == "+" || a == "-") {
        if pos > 0 {
            let left = evaluate(&args[..pos])?;
            let right = evaluate(&args[pos + 1..])?;
            let l: i64 = left.parse().map_err(|_| format!("non-integer argument: {left}"))?;
            let r: i64 = right.parse().map_err(|_| format!("non-integer argument: {right}"))?;

            return Ok(if args[pos] == "+" {
                // Safety: check for integer overflow
                l.checked_add(r)
                    .ok_or_else(|| "integer overflow".to_string())?
            } else {
                l.checked_sub(r)
                    .ok_or_else(|| "integer overflow".to_string())?
            }
            .to_string());
        }
    }

    // Handle * / %
    for op in &["*", "/", "%"] {
        if let Some(pos) = args.iter().rposition(|a| a == *op) {
            let left = evaluate(&args[..pos])?;
            let right = evaluate(&args[pos + 1..])?;
            let l: i64 = left.parse().map_err(|_| format!("non-integer argument: {left}"))?;
            let r: i64 = right.parse().map_err(|_| format!("non-integer argument: {right}"))?;

            return Ok(match *op {
                "*" => l.checked_mul(r).ok_or_else(|| "integer overflow".to_string())?,
                "/" => {
                    if r == 0 {
                        return Err("division by zero".to_string());
                    }
                    l.checked_div(r).ok_or_else(|| "integer overflow".to_string())?
                }
                "%" => {
                    if r == 0 {
                        return Err("division by zero".to_string());
                    }
                    l % r
                }
                _ => unreachable!(),
            }
            .to_string());
        }
    }

    // Handle match / :
    if args.len() >= 3
        && (args[1] == "match" || args[1] == ":") {
            let string = &args[0];
            let pattern = &args[2];
            // Simple regex match
            match regex::Regex::new(pattern) {
                Ok(re) => {
                    if let Some(m) = re.find(string) {
                        return Ok(m.as_str().to_string());
                    } else {
                        return Ok("0".to_string());
                    }
                }
                Err(e) => return Err(format!("invalid pattern: {e}")),
            }
        }

    // Handle length
    if args.len() == 2 && args[0] == "length" {
        return Ok(args[1].len().to_string());
    }

    // Handle substr
    if args.len() == 4 && args[0] == "substr" {
        let string = &args[1];
        let pos: usize = args[2].parse().unwrap_or(1);
        let len: usize = args[3].parse().unwrap_or(0);
        let start = pos.saturating_sub(1);
        let chars: Vec<char> = string.chars().collect();
        let end = (start + len).min(chars.len());
        let result: String = chars[start..end].iter().collect();
        return Ok(result);
    }

    Err(format!("syntax error: {}", args.join(" ")))
}
