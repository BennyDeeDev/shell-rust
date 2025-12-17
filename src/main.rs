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
        let (cmd_str, args_string) = input.split_once(' ').unwrap_or((input, ""));

        match parse_builtin(cmd_str) {
            Some(Builtin::Echo) => {
                println!("{args_string}")
            }
            Some(Builtin::Exit) => {
                break;
            }
            Some(Builtin::Type) => match parse_builtin(args_string) {
                Some(_) => println!("{args_string} is a shell builtin"),
                None => {
                    let exe = find_executable_in_path(args_string);

                    match exe {
                        Some(e) => println!("{} is {}", args_string, e.display()),
                        None => println!("{}: not found", args_string),
                    }
                }
            },
            Some(Builtin::Pwd) => {
                let cwd = env::current_dir().unwrap();

                println!("{}", cwd.display())
            }
            Some(Builtin::Cd) => {
                if args_string == "~" {
                    let home = env::var_os("HOME").unwrap();
                    if std::env::set_current_dir(&home).is_err() {
                        println!("cd: {args_string}: No such file or directory");
                        continue;
                    }
                } else if let Some(rest) = args_string.strip_prefix("~/") {
                    let home = env::var_os("HOME").unwrap();
                    let mut path = PathBuf::from(home);
                    path.push(rest);
                    if std::env::set_current_dir(&path).is_err() {
                        println!("cd: {args_string}: No such file or directory");
                        continue;
                    }
                } else if std::env::set_current_dir(args_string).is_err() {
                    println!("cd: {args_string}: No such file or directory");
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

                let status = Command::new(exe)
                    .arg0(cmd_str)
                    .args(args_string.split_whitespace())
                    .status();

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
