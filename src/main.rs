#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("failed to read line");

        let input = input.trim();
        if input == "exit" {
            break;
        }

        match input {
            s if s.starts_with("echo") => {
                if let Some(rest) = s.strip_prefix("echo ") {
                    println!("{rest}")
                }
            }
            _ => println!("{input}: command not found"),
        }
    }
}
