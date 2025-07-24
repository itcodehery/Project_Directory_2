use std::path::PathBuf;

mod parser;

use parser::parse_command;
struct State {
    state: Option<PathBuf>,
}
fn main() {
    let current_state: State = State { state: None };
    println!("\x1B[2J\x1B[1;1H");
    println!("------------------------");
    println!("Welcome to DIR2");
    println!("------------------------");
    println!("Current State: {:?}", current_state.state);
    command_handler();
}

fn command_handler() {
    loop {
        let mut command: String = String::new();
        eprint!("DIR2>");
        std::io::stdin().read_line(&mut command).unwrap();
        let command: String = command.trim().to_string();
        if command.to_uppercase() == "EXIT" || command.to_uppercase() == "/E" {
            break;
        }
        if command.to_uppercase() == "CLS" || command.to_uppercase() == "/C" {
            println!("\x1B[2J\x1B[1;1H");
            continue;
        }
        if command.is_empty() {
            continue;
        }
        let tokens = parse_command(&command);
        println!("{:?}", tokens);
    }
}
