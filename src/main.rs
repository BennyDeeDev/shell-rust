#[allow(unused_imports)]
use std::io::{self, Write};

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
                None => println!("{rest}: not found"),
            },
            _ => println!("{cmd_str}: command not found"),
        }
    }
}
