#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::{
    env,
    fs::{self},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::Command,
};

enum Builtin {
    Echo,
    Exit,
    Type,
    Pwd,
    Cd,
}

fn parse_builtin(name: &str) -> Option<Builtin> {
    match name {
        "echo" => Some(Builtin::Echo),
        "type" => Some(Builtin::Type),
        "exit" => Some(Builtin::Exit),
        "pwd" => Some(Builtin::Pwd),
        "cd" => Some(Builtin::Cd),
        _ => None,
    }
}

fn find_executable_in_path(name: &str) -> Option<PathBuf> {
    if let Some(path) = env::var_os("PATH") {
        for dir in env::split_paths(&path) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let binary_name = entry.file_name();
                    let meta = match fs::metadata(entry.path()) {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    let perms = meta.permissions();
                    if binary_name == name && (perms.mode() & 0o111) != 0 {
                        return Some(entry.path());
                    }
                }
            }
        }
    }

    None
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("failed to read line");

        let input = input.trim();
        let (cmd_str, args_str) = input.split_once(' ').unwrap_or((input, ""));

        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut single_quoted = false;
        let mut double_quoted = false;
        let mut backslash_escaped = false;

        for c in args_str.chars() {
            match (backslash_escaped, c) {
                (true, ch) => {
                    current_arg.push(ch);
                    backslash_escaped = false;
                }
                (false, '\\') if !double_quoted => {
                    backslash_escaped = true;
                }
                (false, '\'') => {
                    if !double_quoted {
                        single_quoted = !single_quoted;
                    } else {
                        current_arg.push(c);
                    }
                }
                (false, '\"') => {
                    double_quoted = !double_quoted;
                }
                (false, ' ') if !double_quoted && !single_quoted => {
                    if !current_arg.is_empty() {
                        args.push(current_arg);
                        current_arg = String::new();
                    }
                }
                (false, ch) => current_arg.push(ch),
            }
        }

        if !current_arg.is_empty() {
            args.push(current_arg);
        }

        let args_str = args.join(" ");

        match parse_builtin(cmd_str) {
            Some(Builtin::Echo) => {
                println!("{args_str}")
            }
            Some(Builtin::Exit) => {
                break;
            }
            Some(Builtin::Type) => match parse_builtin(&args_str) {
                Some(_) => println!("{args_str} is a shell builtin"),
                None => {
                    let exe = find_executable_in_path(&args_str);

                    match exe {
                        Some(e) => println!("{} is {}", args_str, e.display()),
                        None => println!("{}: not found", args_str),
                    }
                }
            },
            Some(Builtin::Pwd) => {
                let cwd = env::current_dir().unwrap();

                println!("{}", cwd.display())
            }
            Some(Builtin::Cd) => {
                if args_str == "~" {
                    let home = env::var_os("HOME").unwrap();
                    if std::env::set_current_dir(&home).is_err() {
                        println!("cd: {args_str}: No such file or directory");
                        continue;
                    }
                } else if let Some(rest) = args_str.strip_prefix("~/") {
                    let home = env::var_os("HOME").unwrap();
                    let mut path = PathBuf::from(home);
                    path.push(rest);
                    if std::env::set_current_dir(&path).is_err() {
                        println!("cd: {args_str}: No such file or directory");
                        continue;
                    }
                } else if std::env::set_current_dir(&args_str).is_err() {
                    println!("cd: {args_str}: No such file or directory");
                    continue;
                }
            }
            _ => {
                let exe = match find_executable_in_path(cmd_str) {
                    Some(e) => e,
                    None => {
                        println!("{cmd_str}: not found");
                        continue;
                    }
                };

                let status = Command::new(exe).arg0(cmd_str).args(&args).status();

                match status {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("{cmd_str}: {e}");
                        continue;
                    }
                };
            }
        }
    }
}
