#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, fs, os::unix::fs::PermissionsExt};

enum Builtin {
    Echo,
    Exit,
    Type,
}

fn parse_builtin(name: &str) -> Option<Builtin> {
    match name {
        "echo" => Some(Builtin::Echo),
        "type" => Some(Builtin::Type),
        "exit" => Some(Builtin::Exit),
        _ => None,
    }
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
        let (cmd_str, rest) = input.split_once(' ').unwrap_or((input, ""));

        match parse_builtin(cmd_str) {
            Some(Builtin::Echo) => {
                println!("{rest}")
            }
            Some(Builtin::Exit) => {
                break;
            }
            Some(Builtin::Type) => match parse_builtin(rest) {
                Some(_) => println!("{rest} is a shell builtin"),
                None => {
                    let mut found = false;

                    if let Some(path) = env::var_os("PATH") {
                        'outer: for dir in env::split_paths(&path) {
                            if let Ok(entries) = fs::read_dir(dir) {
                                for entry in entries.flatten() {
                                    let binary_name = entry.file_name();
                                    let meta = match fs::metadata(entry.path()) {
                                        Ok(m) => m,
                                        Err(_) => continue,
                                    };

                                    let perms = meta.permissions();
                                    if binary_name == rest && (perms.mode() & 0o111) != 0 {
                                        println!("{} is {}", rest, entry.path().display());
                                        found = true;
                                        break 'outer;
                                    }
                                }
                            }
                        }
                    }

                    if !found {
                        println!("{rest}: not found")
                    }
                }
            },
            _ => {
                println!("{input}: command not found")
            }
        }
    }
}
