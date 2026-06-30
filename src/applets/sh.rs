/*
 * <purpose of file>
 *
 * Copyright (C) 2026 by Anandkumar  <Truchip >
 *
 * Licensed under MIT
 */
//! sh — minimal POSIX-like shell
//!
//! Supports: command execution, pipes, redirections (>, >>, <),
//! environment variables, $?, cd, exit, export, unset, if/then/else/fi,
//! while/do/done, for/do/done, comments, quoting, and script execution.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::os::unix::io::FromRawFd;
use std::os::unix::process::CommandExt;
use std::process::{self, Command, Stdio};

pub fn run(args: &[String]) -> i32 {
    let mut interactive = false;
    let mut command_string: Option<String> = None;
    let mut script_file: Option<String> = None;
    let mut script_args: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-c" => {
                i += 1;
                if i < args.len() {
                    command_string = Some(args[i].clone());
                }
            }
            "-i" => interactive = true,
            "--help" | "-h" => {
                eprintln!("Usage: sh [-c command] [-i] [script [args...]]");
                return 0;
            }
            _ if script_file.is_none() && !args[i].starts_with('-') => {
                script_file = Some(args[i].clone());
                script_args = args[i + 1..].to_vec();
                break;
            }
            _ => {}
        }
        i += 1;
    }

    let mut shell = Shell::new();

    // Detect login shell: argv[0] starts with '-' (e.g. "-sh")
    let is_login = std::env::args()
        .next()
        .is_some_and(|a| a.starts_with('-'));

    // Source /etc/profile for login shells
    if is_login
        && std::path::Path::new("/etc/profile").exists() {
            shell.execute_file("/etc/profile");
        }

    // sh -c "command"
    if let Some(cmd) = command_string {
        return shell.execute_string(&cmd);
    }

    // sh script.sh [args]
    if let Some(ref path) = script_file {
        shell.set_positional_params(&script_args);
        return shell.execute_file(path);
    }

    // Interactive or stdin
    if !interactive {
        interactive = unsafe { libc::isatty(0) } != 0;
    }

    shell.run_interactive(interactive)
}

struct Shell {
    last_exit: i32,
    variables: HashMap<String, String>,
    running: bool,
}

impl Shell {
    fn new() -> Self {
        Shell {
            last_exit: 0,
            variables: HashMap::new(),
            running: true,
        }
    }

    fn set_positional_params(&mut self, params: &[String]) {
        self.variables
            .insert("#".to_string(), params.len().to_string());
        for (i, p) in params.iter().enumerate() {
            self.variables.insert((i + 1).to_string(), p.clone());
        }
    }

    fn run_interactive(&mut self, interactive: bool) -> i32 {
        let stdin = io::stdin();

        if interactive {
            self.print_prompt();
        }

        let mut lines = String::new();
        for line_result in stdin.lock().lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => break,
            };

            lines.push_str(&line);

            // Handle line continuation
            if lines.ends_with('\\') {
                lines.pop();
                lines.push('\n');
                if interactive {
                    eprint!("> ");
                    let _ = io::stderr().flush();
                }
                continue;
            }

            self.execute_string(&lines);
            lines.clear();

            if !self.running {
                break;
            }

            if interactive {
                self.print_prompt();
            }
        }

        self.last_exit
    }

    fn print_prompt(&self) {
        let ps1 = env::var("PS1").unwrap_or_else(|_| {
            if unsafe { libc::getuid() } == 0 {
                "# ".to_string()
            } else {
                "$ ".to_string()
            }
        });
        let expanded = self.expand_prompt(&ps1);
        eprint!("{expanded}");
        let _ = io::stderr().flush();
    }

    /// Expand PS1 prompt escape sequences: \u, \h, \H, \w, \W, \$, \\, \n
    fn expand_prompt(&self, ps1: &str) -> String {
        let mut result = String::with_capacity(ps1.len());
        let mut chars = ps1.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('u') => {
                        result.push_str(&env::var("USER").unwrap_or_else(|_| "?".into()));
                    }
                    Some('h') => {
                        let host = self.get_hostname();
                        // \h = short hostname (up to first '.')
                        if let Some(short) = host.split('.').next() {
                            result.push_str(short);
                        }
                    }
                    Some('H') => {
                        result.push_str(&self.get_hostname());
                    }
                    Some('w') => {
                        let cwd = env::var("PWD")
                            .or_else(|_| env::current_dir().map(|p| p.to_string_lossy().to_string()))
                            .unwrap_or_else(|_| "?".into());
                        let home = env::var("HOME").unwrap_or_default();
                        if !home.is_empty() && cwd.starts_with(&home) {
                            result.push('~');
                            result.push_str(&cwd[home.len()..]);
                        } else {
                            result.push_str(&cwd);
                        }
                    }
                    Some('W') => {
                        let cwd = env::var("PWD")
                            .or_else(|_| env::current_dir().map(|p| p.to_string_lossy().to_string()))
                            .unwrap_or_else(|_| "?".into());
                        let home = env::var("HOME").unwrap_or_default();
                        if cwd == home {
                            result.push('~');
                        } else if cwd == "/" {
                            result.push('/');
                        } else {
                            result.push_str(
                                std::path::Path::new(&cwd)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("?"),
                            );
                        }
                    }
                    Some('$') => {
                        if unsafe { libc::getuid() } == 0 {
                            result.push('#');
                        } else {
                            result.push('$');
                        }
                    }
                    Some('\\') => result.push('\\'),
                    Some('n') => result.push('\n'),
                    Some('[') | Some(']') => {} // ignore \[ \] (non-printing delimiters)
                    Some(other) => {
                        result.push('\\');
                        result.push(other);
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(c);
            }
        }
        result
    }

    fn get_hostname(&self) -> String {
        fs::read_to_string("/etc/hostname")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| {
                unsafe {
                    let mut utsname: libc::utsname = std::mem::zeroed();
                    libc::uname(&mut utsname);
                    std::ffi::CStr::from_ptr(utsname.nodename.as_ptr())
                        .to_string_lossy()
                        .to_string()
                }
            })
    }

    fn execute_file(&mut self, path: &str) -> i32 {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("sh: {path}: {e}");
                return 127;
            }
        };
        self.execute_string(&content)
    }

    fn execute_string(&mut self, input: &str) -> i32 {
        let lines = split_lines(input);
        let mut i = 0;
        while i < lines.len() && self.running {
            let line = lines[i].trim();
            i += 1;

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Handle control structures
            if line == "if" || line.starts_with("if ") {
                i = self.handle_if(&lines, i - 1);
                continue;
            }
            if line == "while" || line.starts_with("while ") {
                i = self.handle_while(&lines, i - 1);
                continue;
            }
            if line == "for" || line.starts_with("for ") {
                i = self.handle_for(&lines, i - 1);
                continue;
            }

            // Handle semicolon-separated commands
            for cmd in split_on_semicolons(line) {
                let cmd = cmd.trim();
                if !cmd.is_empty() {
                    self.execute_command_line(cmd);
                }
                if !self.running {
                    break;
                }
            }
        }
        self.last_exit
    }

    fn handle_if(&mut self, lines: &[String], start: usize) -> usize {
        // Parse: if COND; then BODY [; elif COND; then BODY]* [; else BODY]; fi
        let mut i = start;
        let mut depth = 0;
        let mut blocks: Vec<String> = Vec::new();
        let mut current_block = String::new();

        // Collect all lines between if..fi
        while i < lines.len() {
            let line = lines[i].trim();
            i += 1;

            if line.starts_with("if ") || line == "if" {
                if depth > 0 {
                    current_block.push_str(line);
                    current_block.push('\n');
                }
                depth += 1;
                if depth == 1 {
                    // Extract condition after "if"
                    let cond = line.strip_prefix("if ").unwrap_or("").trim();
                    // Check if "then" is on same line after ";"
                    if let Some((cond_part, rest)) = cond.split_once("; then") {
                        blocks.push(format!("COND:{}", cond_part.trim()));
                        if !rest.trim().is_empty() {
                            current_block.push_str(rest.trim());
                            current_block.push('\n');
                        }
                    } else if cond.ends_with("; then") || cond.ends_with(";then") {
                        let c = cond.trim_end_matches("then").trim_end_matches(';').trim();
                        blocks.push(format!("COND:{c}"));
                    } else {
                        blocks.push(format!("COND:{cond}"));
                    }
                    continue;
                }
            }

            if depth == 1 && line == "then" {
                continue;
            }

            if depth == 1 && (line.starts_with("elif ") || line == "elif") {
                // Save current body block
                blocks.push(format!("BODY:{}", current_block.trim()));
                current_block.clear();
                let cond = line.strip_prefix("elif ").unwrap_or("").trim();
                if let Some((cond_part, rest)) = cond.split_once("; then") {
                    blocks.push(format!("COND:{}", cond_part.trim()));
                    if !rest.trim().is_empty() {
                        current_block.push_str(rest.trim());
                        current_block.push('\n');
                    }
                } else {
                    blocks.push(format!("COND:{cond}"));
                }
                continue;
            }

            if depth == 1 && line == "else" {
                blocks.push(format!("BODY:{}", current_block.trim()));
                current_block.clear();
                blocks.push("ELSE".to_string());
                continue;
            }

            if line == "fi" {
                depth -= 1;
                if depth == 0 {
                    blocks.push(format!("BODY:{}", current_block.trim()));
                    break;
                }
            }

            current_block.push_str(line);
            current_block.push('\n');
        }

        // Evaluate: COND, BODY pairs, optional ELSE, BODY
        let mut bi = 0;
        let mut executed = false;
        while bi < blocks.len() {
            if blocks[bi].starts_with("COND:") {
                let cond = &blocks[bi]["COND:".len()..];
                self.execute_string(cond);
                bi += 1;
                if bi < blocks.len() && blocks[bi].starts_with("BODY:") {
                    if self.last_exit == 0 && !executed {
                        let body = &blocks[bi]["BODY:".len()..];
                        self.execute_string(body);
                        executed = true;
                    }
                    bi += 1;
                }
            } else if blocks[bi] == "ELSE" {
                bi += 1;
                if bi < blocks.len() && blocks[bi].starts_with("BODY:") {
                    if !executed {
                        let body = &blocks[bi]["BODY:".len()..];
                        self.execute_string(body);
                    }
                    bi += 1;
                }
            } else {
                bi += 1;
            }
        }
        i
    }

    fn handle_while(&mut self, lines: &[String], start: usize) -> usize {
        let mut i = start;
        let mut depth = 0;
        let mut cond_str = String::new();
        let mut body_str = String::new();
        let mut in_body = false;

        while i < lines.len() {
            let line = lines[i].trim();
            i += 1;

            if line.starts_with("while ") || line == "while" {
                depth += 1;
                if depth == 1 {
                    let rest = line.strip_prefix("while ").unwrap_or("").trim();
                    if let Some((c, _)) = rest.split_once("; do") {
                        cond_str = c.trim().to_string();
                    } else {
                        cond_str = rest.to_string();
                    }
                    continue;
                }
            }

            if depth == 1 && (line == "do" || line.ends_with("; do")) {
                in_body = true;
                continue;
            }

            if line == "done" {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }

            if depth >= 1 {
                if in_body {
                    body_str.push_str(line);
                    body_str.push('\n');
                } else {
                    cond_str.push_str(line);
                    cond_str.push('\n');
                }
            }
        }

        // Execute while loop
        loop {
            self.execute_string(&cond_str);
            if self.last_exit != 0 {
                break;
            }
            self.execute_string(&body_str);
        }
        i
    }

    fn handle_for(&mut self, lines: &[String], start: usize) -> usize {
        let mut i = start;
        let mut depth = 0;
        let mut var_name = String::new();
        let mut word_list: Vec<String> = Vec::new();
        let mut body_str = String::new();
        let mut in_body = false;

        while i < lines.len() {
            let line = lines[i].trim();
            i += 1;

            if line.starts_with("for ") || line == "for" {
                depth += 1;
                if depth == 1 {
                    // Parse: for VAR in WORDS; do  or  for VAR in WORDS\ndo
                    let rest = line.strip_prefix("for ").unwrap_or("").trim();
                    if let Some((before_do, _)) = rest.split_once("; do") {
                        let parts = before_do.trim();
                        if let Some((var, words)) = parts.split_once(" in ") {
                            var_name = var.trim().to_string();
                            word_list = self.expand_words(words.trim());
                        }
                        in_body = true;
                    } else if let Some((var, words)) = rest.split_once(" in ") {
                        var_name = var.trim().to_string();
                        word_list = self.expand_words(words.trim());
                    } else {
                        var_name = rest.to_string();
                    }
                    continue;
                }
            }

            if depth == 1 && (line == "do" || line.ends_with("; do")) {
                in_body = true;
                continue;
            }

            if line == "done" {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }

            if depth >= 1 && in_body {
                body_str.push_str(line);
                body_str.push('\n');
            }
        }

        // Execute for loop
        for word in &word_list {
            env::set_var(&var_name, word);
            self.execute_string(&body_str);
        }
        i
    }

    fn expand_words(&self, s: &str) -> Vec<String> {
        
        tokenize(&self.expand_variables(s))
    }

    fn execute_command_line(&mut self, line: &str) {
        // Handle && and ||
        let segments = split_logical(line);
        let mut i = 0;
        while i < segments.len() {
            let (op, cmd) = &segments[i];
            i += 1;

            match op.as_str() {
                "" | ";" => {
                    self.execute_pipeline(cmd.trim());
                }
                "&&" => {
                    if self.last_exit == 0 {
                        self.execute_pipeline(cmd.trim());
                    }
                }
                "||" => {
                    if self.last_exit != 0 {
                        self.execute_pipeline(cmd.trim());
                    }
                }
                _ => {
                    self.execute_pipeline(cmd.trim());
                }
            }
        }
    }

    fn execute_pipeline(&mut self, line: &str) {
        let commands = split_pipe(line);
        if commands.len() == 1 {
            self.execute_simple(&commands[0]);
            return;
        }

        // Set up pipeline
        let mut prev_read: Option<i32> = None;
        let num_cmds = commands.len();

        let mut children: Vec<Option<process::Child>> = Vec::new();

        for (idx, cmd_str) in commands.iter().enumerate() {
            let is_last = idx == num_cmds - 1;
            let tokens = self.prepare_tokens(cmd_str.trim());
            if tokens.is_empty() {
                continue;
            }

            let (args, redir) = parse_redirections(&tokens);
            if args.is_empty() {
                continue;
            }

            let (pipe_read, pipe_write) = if !is_last {
                let mut fds = [0i32; 2];
                // Create the pipe with O_CLOEXEC so that the raw fds are not
                // leaked into unrelated children (e.g. the producer inheriting
                // its own pipe's read end, which would prevent it from ever
                // seeing a broken pipe). The dup2 that wires up stdin/stdout
                // clears CLOEXEC on the fds that actually need to survive exec.
                if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) } != 0 {
                    eprintln!("sh: pipe failed");
                    self.last_exit = 1;
                    return;
                }
                (Some(fds[0]), Some(fds[1]))
            } else {
                (None, None)
            };

            let stdin_cfg = if let Some(ref path) = redir.stdin_file {
                match fs::File::open(path) {
                    Ok(f) => Stdio::from(f),
                    Err(e) => {
                        eprintln!("sh: {path}: {e}");
                        self.last_exit = 1;
                        return;
                    }
                }
            } else {
                match prev_read {
                    Some(fd) => unsafe { Stdio::from_raw_fd(fd) },
                    None => Stdio::inherit(),
                }
            };

            let stdout_cfg = if let Some((ref path, append)) = redir.stdout_file {
                let f = fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(append)
                    .truncate(!append)
                    .open(path);
                match f {
                    Ok(file) => Stdio::from(file),
                    Err(e) => {
                        eprintln!("sh: {path}: {e}");
                        self.last_exit = 1;
                        return;
                    }
                }
            } else {
                match pipe_write {
                    Some(fd) => unsafe { Stdio::from_raw_fd(fd) },
                    None => Stdio::inherit(),
                }
            };

            let mut cmd = Command::new(&args[0]);
            cmd.args(&args[1..])
                .stdin(stdin_cfg)
                .stdout(stdout_cfg);

            // Restore the default SIGPIPE disposition in children. Rust sets
            // SIGPIPE to SIG_IGN at startup and children inherit it, which would
            // otherwise keep a pipeline producer (e.g. `yes`) alive after its
            // consumer (e.g. `head`) has exited instead of letting it die on a
            // broken pipe.
            unsafe {
                cmd.pre_exec(|| {
                    libc::signal(libc::SIGPIPE, libc::SIG_DFL);
                    Ok(())
                });
            }

            if redir.stderr_to_stdout {
                unsafe {
                    cmd.pre_exec(|| {
                        libc::dup2(1, 2);
                        Ok(())
                    });
                }
            } else if let Some((ref path, append)) = redir.stderr_file {
                let f = fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(append)
                    .truncate(!append)
                    .open(path);
                match f {
                    Ok(file) => { cmd.stderr(file); }
                    Err(e) => {
                        eprintln!("sh: {path}: {e}");
                        self.last_exit = 1;
                        return;
                    }
                }
            }

            let child = cmd.spawn();

            // Close the read end from previous pipe in parent
            if let Some(fd) = prev_read {
                unsafe { libc::close(fd) };
            }

            prev_read = pipe_read;

            match child {
                Ok(c) => children.push(Some(c)),
                Err(e) => {
                    eprintln!("sh: {}: {e}", args[0]);
                    self.last_exit = 127;
                    children.push(None);
                }
            }
        }

        // Wait for all children; exit status is from last command
        for (idx, child) in children.iter_mut().enumerate() {
            if let Some(ref mut c) = child {
                match c.wait() {
                    Ok(status) => {
                        if idx == num_cmds - 1 {
                            self.last_exit = status.code().unwrap_or(1);
                        }
                    }
                    Err(_) => {
                        if idx == num_cmds - 1 {
                            self.last_exit = 1;
                        }
                    }
                }
            }
        }
    }

    fn execute_simple(&mut self, cmd_str: &str) {
        let tokens = self.prepare_tokens(cmd_str);
        if tokens.is_empty() {
            return;
        }

        // Parse redirections
        let (args, redir) = parse_redirections(&tokens);

        if args.is_empty() {
            return;
        }

        // Check for variable assignment: VAR=value
        if args.len() == 1 && args[0].contains('=') && !args[0].starts_with('=') {
            let eq_pos = args[0].find('=').unwrap();
            let var_part = &args[0][..eq_pos];
            if var_part.chars().all(|c| c.is_alphanumeric() || c == '_') {
                let val = &args[0][eq_pos + 1..];
                self.variables.insert(var_part.to_string(), val.to_string());
                env::set_var(var_part, val);
                self.last_exit = 0;
                return;
            }
        }

        // Builtins
        match args[0].as_str() {
            "exit" => {
                let code = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(self.last_exit);
                self.running = false;
                self.last_exit = code;
                return;
            }
            "cd" => {
                let dir = args.get(1).map(|s| s.as_str()).unwrap_or_else(|| {
                    env::var("HOME")
                        .ok()
                        .as_deref()
                        .unwrap_or("/")
                        .to_string()
                        .leak()
                });
                match env::set_current_dir(dir) {
                    Ok(()) => {
                        if let Ok(cwd) = env::current_dir() {
                            env::set_var("PWD", cwd);
                        }
                        self.last_exit = 0;
                    }
                    Err(e) => {
                        eprintln!("sh: cd: {dir}: {e}");
                        self.last_exit = 1;
                    }
                }
                return;
            }
            "export" => {
                for arg in &args[1..] {
                    if let Some((key, val)) = arg.split_once('=') {
                        env::set_var(key, val);
                        self.variables.insert(key.to_string(), val.to_string());
                    } else if let Some(val) = self.variables.get(arg.as_str()) {
                        env::set_var(arg, val);
                    }
                }
                self.last_exit = 0;
                return;
            }
            "unset" => {
                for arg in &args[1..] {
                    env::remove_var(arg);
                    self.variables.remove(arg.as_str());
                }
                self.last_exit = 0;
                return;
            }
            "source" | "." => {
                if let Some(path) = args.get(1) {
                    self.execute_file(path);
                } else {
                    eprintln!("sh: source: filename argument required");
                    self.last_exit = 1;
                }
                return;
            }
            "read" => {
                self.builtin_read(&args[1..]);
                return;
            }
            "set" => {
                if args.len() > 1 && args[1] == "-e" {
                    // Acknowledged but not enforced in this minimal shell
                }
                self.last_exit = 0;
                return;
            }
            "[" | "test" => {
                // Builtin test/[ — delegate to the test applet
                #[cfg(applet_test)]
                {
                    let test_args: Vec<String> = args.clone();
                    self.last_exit = crate::applets::test::run(&test_args[1..]);
                }
                #[cfg(not(applet_test))]
                {
                    eprintln!("sh: test: applet not compiled in");
                    self.last_exit = 127;
                }
                return;
            }
            "exec" => {
                if args.len() > 1 {
                    let c_prog = std::ffi::CString::new(args[1].as_str()).unwrap();
                    let c_args: Vec<std::ffi::CString> = args[1..]
                        .iter()
                        .map(|a| std::ffi::CString::new(a.as_str()).unwrap())
                        .collect();
                    let c_arg_ptrs: Vec<*const libc::c_char> = c_args
                        .iter()
                        .map(|a| a.as_ptr())
                        .chain(std::iter::once(std::ptr::null()))
                        .collect();
                    unsafe { libc::execvp(c_prog.as_ptr(), c_arg_ptrs.as_ptr()) };
                    eprintln!("sh: exec: {}: {}", args[1], io::Error::last_os_error());
                    self.last_exit = 126;
                }
                return;
            }
            _ => {}
        }

        // External command
        let mut cmd = Command::new(&args[0]);
        cmd.args(&args[1..]);

        if let Some(ref path) = redir.stdin_file {
            match fs::File::open(path) {
                Ok(f) => { cmd.stdin(f); }
                Err(e) => {
                    eprintln!("sh: {path}: {e}");
                    self.last_exit = 1;
                    return;
                }
            }
        }

        if let Some((ref path, append)) = redir.stdout_file {
            let f = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(append)
                .truncate(!append)
                .open(path);
            match f {
                Ok(file) => { cmd.stdout(file); }
                Err(e) => {
                    eprintln!("sh: {path}: {e}");
                    self.last_exit = 1;
                    return;
                }
            }
        }

        if redir.stderr_to_stdout {
            unsafe {
                cmd.pre_exec(|| {
                    libc::dup2(1, 2);
                    Ok(())
                });
            }
        } else if let Some((ref path, append)) = redir.stderr_file {
            let f = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(append)
                .truncate(!append)
                .open(path);
            match f {
                Ok(file) => { cmd.stderr(file); }
                Err(e) => {
                    eprintln!("sh: {path}: {e}");
                    self.last_exit = 1;
                    return;
                }
            }
        }

        match cmd.status() {
            Ok(status) => {
                self.last_exit = status.code().unwrap_or(1);
            }
            Err(e) => {
                eprintln!("sh: {}: {e}", args[0]);
                self.last_exit = 127;
            }
        }
    }

    fn prepare_tokens(&self, cmd: &str) -> Vec<String> {
        let expanded = self.expand_variables(cmd);
        tokenize(&expanded)
    }

    fn expand_variables(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        let mut in_single_quote = false;

        while let Some(c) = chars.next() {
            if c == '\'' && !in_single_quote {
                in_single_quote = true;
                result.push(c);
                continue;
            }
            if c == '\'' && in_single_quote {
                in_single_quote = false;
                result.push(c);
                continue;
            }
            if in_single_quote {
                result.push(c);
                continue;
            }

            if c == '$' {
                match chars.peek() {
                    Some(&'{') => {
                        chars.next();
                        let mut var_name = String::new();
                        for vc in chars.by_ref() {
                            if vc == '}' {
                                break;
                            }
                            var_name.push(vc);
                        }
                        result.push_str(&self.get_variable(&var_name));
                    }
                    Some(&'(') => {
                        chars.next();
                        let mut depth = 1;
                        let mut sub_cmd = String::new();
                        for vc in chars.by_ref() {
                            if vc == '(' {
                                depth += 1;
                            } else if vc == ')' {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            sub_cmd.push(vc);
                        }
                        // Command substitution
                        let output = Command::new("/bin/sh")
                            .args(["-c", &sub_cmd])
                            .output();
                        if let Ok(out) = output {
                            let s = String::from_utf8_lossy(&out.stdout);
                            result.push_str(s.trim_end_matches('\n'));
                        }
                    }
                    Some(&'?') => {
                        chars.next();
                        result.push_str(&self.last_exit.to_string());
                    }
                    Some(&'$') => {
                        chars.next();
                        result.push_str(&unsafe { libc::getpid() }.to_string());
                    }
                    Some(&'#') => {
                        chars.next();
                        let val = self.variables.get("#").map(|s| s.as_str()).unwrap_or("0");
                        result.push_str(val);
                    }
                    Some(&'0') => {
                        chars.next();
                        result.push_str("sh");
                    }
                    Some(&c2) if c2.is_ascii_digit() => {
                        chars.next();
                        let key = c2.to_string();
                        let val = self
                            .variables
                            .get(&key)
                            .map(|s| s.as_str())
                            .unwrap_or("");
                        result.push_str(val);
                    }
                    Some(&c2) if c2.is_alphabetic() || c2 == '_' => {
                        let mut var_name = String::new();
                        while let Some(&vc) = chars.peek() {
                            if vc.is_alphanumeric() || vc == '_' {
                                var_name.push(vc);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        result.push_str(&self.get_variable(&var_name));
                    }
                    _ => result.push('$'),
                }
            } else if c == '`' {
                // Backtick command substitution
                let mut sub_cmd = String::new();
                for vc in chars.by_ref() {
                    if vc == '`' {
                        break;
                    }
                    sub_cmd.push(vc);
                }
                let output = Command::new("/bin/sh")
                    .args(["-c", &sub_cmd])
                    .output();
                if let Ok(out) = output {
                    let s = String::from_utf8_lossy(&out.stdout);
                    result.push_str(s.trim_end_matches('\n'));
                }
            } else {
                result.push(c);
            }
        }
        result
    }

    fn get_variable(&self, name: &str) -> String {
        // Check shell variables first, then environment
        if let Some(val) = self.variables.get(name) {
            val.clone()
        } else {
            env::var(name).unwrap_or_default()
        }
    }

    fn builtin_read(&mut self, args: &[String]) {
        let var_name = args.first().map(|s| s.as_str()).unwrap_or("REPLY");
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => {
                self.last_exit = 1;
            }
            Ok(_) => {
                let val = line.trim_end_matches('\n').to_string();
                self.variables.insert(var_name.to_string(), val.clone());
                env::set_var(var_name, &val);
                self.last_exit = 0;
            }
            Err(_) => {
                self.last_exit = 1;
            }
        }
    }
}

/// Split input into logical lines (handling multi-line constructs flattened by ;)
fn split_lines(input: &str) -> Vec<String> {
    input.lines().map(|l| l.to_string()).collect()
}

/// Tokenize a command string, respecting quotes
struct Redirections {
    stdout_file: Option<(String, bool)>, // (path, append)
    stdin_file: Option<String>,
    stderr_file: Option<(String, bool)>, // (path, append)
    stderr_to_stdout: bool,              // 2>&1
}

fn parse_redirections(tokens: &[String]) -> (Vec<String>, Redirections) {
    let mut args: Vec<String> = Vec::new();
    let mut redir = Redirections {
        stdout_file: None,
        stdin_file: None,
        stderr_file: None,
        stderr_to_stdout: false,
    };

    let mut i = 0;
    while i < tokens.len() {
        match tokens[i].as_str() {
            ">>" => {
                i += 1;
                if i < tokens.len() {
                    redir.stdout_file = Some((tokens[i].clone(), true));
                }
            }
            ">" => {
                i += 1;
                if i < tokens.len() {
                    redir.stdout_file = Some((tokens[i].clone(), false));
                }
            }
            "2>&1" => {
                redir.stderr_to_stdout = true;
            }
            "2>" => {
                i += 1;
                if i < tokens.len() {
                    redir.stderr_file = Some((tokens[i].clone(), false));
                }
            }
            "2>>" => {
                i += 1;
                if i < tokens.len() {
                    redir.stderr_file = Some((tokens[i].clone(), true));
                }
            }
            "<" => {
                i += 1;
                if i < tokens.len() {
                    redir.stdin_file = Some(tokens[i].clone());
                }
            }
            _ => {
                if tokens[i].starts_with("2>>") {
                    redir.stderr_file = Some((tokens[i][3..].to_string(), true));
                } else if tokens[i].starts_with("2>&1") {
                    redir.stderr_to_stdout = true;
                } else if tokens[i].starts_with("2>") {
                    redir.stderr_file = Some((tokens[i][2..].to_string(), false));
                } else if tokens[i].starts_with(">>") {
                    redir.stdout_file = Some((tokens[i][2..].to_string(), true));
                } else if tokens[i].starts_with('>') {
                    redir.stdout_file = Some((tokens[i][1..].to_string(), false));
                } else if tokens[i].starts_with('<') {
                    redir.stdin_file = Some(tokens[i][1..].to_string());
                } else {
                    args.push(tokens[i].clone());
                }
            }
        }
        i += 1;
    }
    (args, redir)
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    for c in input.chars() {
        if escape_next {
            if in_double_quote {
                // In double quotes, only \$ \" \\ \` \newline are special
                match c {
                    '$' | '"' | '\\' | '`' => current.push(c),
                    '\n' => {} // line continuation
                    _ => {
                        current.push('\\');
                        current.push(c);
                    }
                }
            } else {
                current.push(c);
            }
            escape_next = false;
            continue;
        }

        match c {
            '\\' if !in_single_quote => {
                escape_next = true;
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
                // Don't push the quote character
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
                // Don't push the quote character
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Split on unquoted semicolons
fn split_on_semicolons(input: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;

    for c in input.chars() {
        if escape {
            current.push('\\');
            current.push(c);
            escape = false;
            continue;
        }
        match c {
            '\\' if !in_single => escape = true,
            '\'' if !in_double => {
                in_single = !in_single;
                current.push(c);
            }
            '"' if !in_single => {
                in_double = !in_double;
                current.push(c);
            }
            ';' if !in_single && !in_double => {
                segments.push(std::mem::take(&mut current));
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments
}

/// Split on unquoted pipes (not ||)
fn split_pipe(input: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if escape {
            current.push('\\');
            current.push(c);
            escape = false;
            continue;
        }
        match c {
            '\\' if !in_single => escape = true,
            '\'' if !in_double => {
                in_single = !in_single;
                current.push(c);
            }
            '"' if !in_single => {
                in_double = !in_double;
                current.push(c);
            }
            '|' if !in_single && !in_double => {
                if chars.peek() == Some(&'|') {
                    // || operator, not a pipe
                    current.push('|');
                    current.push('|');
                    chars.next();
                } else {
                    segments.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments
}

/// Split on && and || operators
fn split_logical(input: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut current_op = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if escape {
            current.push('\\');
            current.push(c);
            escape = false;
            continue;
        }
        match c {
            '\\' if !in_single => escape = true,
            '\'' if !in_double => {
                in_single = !in_single;
                current.push(c);
            }
            '"' if !in_single => {
                in_double = !in_double;
                current.push(c);
            }
            '&' if !in_single && !in_double && chars.peek() == Some(&'&') => {
                chars.next();
                result.push((current_op, std::mem::take(&mut current)));
                current_op = "&&".to_string();
            }
            '|' if !in_single && !in_double && chars.peek() == Some(&'|') => {
                chars.next();
                result.push((current_op, std::mem::take(&mut current)));
                current_op = "||".to_string();
            }
            _ => current.push(c),
        }
    }
    result.push((current_op, current));
    result
}
