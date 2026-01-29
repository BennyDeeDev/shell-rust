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

#[derive(Copy, Clone)]
enum QuoteMode {
    Single,
    Double,
}

#[derive(Copy, Clone)]
enum QuoteModeState {
    Normal,
    Escaped,
    Quoted(QuoteMode),
    QuotedEscaped(QuoteMode),
}

fn parse_shell_tokens(str: &str) -> Vec<String> {
    let mut str_vec = Vec::new();
    let mut current_str = String::new();
    let mut quote_mode_state = QuoteModeState::Normal;

    for c in str.chars() {
        match (quote_mode_state, c) {
            (QuoteModeState::Escaped, ch) => {
                quote_mode_state = QuoteModeState::Normal;
                current_str.push(ch);
            }
            (QuoteModeState::QuotedEscaped(QuoteMode::Single), ch) => {
                quote_mode_state = QuoteModeState::Quoted(QuoteMode::Single);
                current_str.push(ch);
            }
            (QuoteModeState::QuotedEscaped(QuoteMode::Double), ch) => {
                quote_mode_state = QuoteModeState::Quoted(QuoteMode::Double);
                if matches!(ch, '"' | '\\' | '$' | '`') {
                    current_str.push(ch);
                } else {
                    current_str.push('\\');
                    current_str.push(ch);
                }
            }
            (QuoteModeState::Normal, '\\') => {
                quote_mode_state = QuoteModeState::Escaped;
            }
            (QuoteModeState::Quoted(QuoteMode::Double), '\\') => {
                quote_mode_state = QuoteModeState::QuotedEscaped(QuoteMode::Double);
            }
            (QuoteModeState::Quoted(QuoteMode::Single), '\\') => {
                current_str.push(c);
            }
            (QuoteModeState::Normal, '\'') => {
                quote_mode_state = QuoteModeState::Quoted(QuoteMode::Single);
            }
            (QuoteModeState::Normal, '\"') => {
                quote_mode_state = QuoteModeState::Quoted(QuoteMode::Double);
            }
            (QuoteModeState::Quoted(QuoteMode::Single), '\'') => {
                quote_mode_state = QuoteModeState::Normal;
            }
            (QuoteModeState::Quoted(QuoteMode::Double), '\"') => {
                quote_mode_state = QuoteModeState::Normal;
            }
            (QuoteModeState::Normal, ' ') => {
                if !current_str.is_empty() {
                    str_vec.push(current_str);
                    current_str = String::new();
                }
            }
            (QuoteModeState::Normal, ch) => {
                current_str.push(ch);
            }
            (QuoteModeState::Quoted(QuoteMode::Single), ch) => {
                current_str.push(ch);
            }
            (QuoteModeState::Quoted(QuoteMode::Double), ch) => {
                current_str.push(ch);
            }
        }
    }

    if !current_str.is_empty() {
        str_vec.push(current_str);
    }

    str_vec
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
        let parsed_shell_tokens = parse_shell_tokens(input);
        let (cmd_str, args) = match parsed_shell_tokens.split_first() {
            Some(parts) => parts,
            None => continue,
        };

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

                let status = Command::new(exe).arg0(cmd_str).args(args).status();

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
