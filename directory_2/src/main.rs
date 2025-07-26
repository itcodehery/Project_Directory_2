use std::path::PathBuf;

mod parser;
mod file_system_state;
mod filesystem;
mod indexing;
mod commands;
mod config;
mod search;
mod favorites;

use parser::parse_command;
use file_system_state::FileSystemState;
use crate::commands::execute_command;

struct State {
    state: Option<PathBuf>,
}

fn main() {
    // Dependency Injection of the State Variable
    let current_file_sys_state: FileSystemState = FileSystemState::new();
    println!("\x1B[2J\x1B[1;1H");
    terminal_boilerplate(&current_file_sys_state);
    command_handler(&current_file_sys_state);
}

fn terminal_boilerplate(sys_state: &FileSystemState) {
    println!("------------------------");
    println!("Welcome to DIR2");
    println!("------------------------");
    println!("Current State: {:?}", sys_state.get_current_state());
    println!("Current Directory: {:?}", sys_state.get_current_path());
}

fn command_handler(sys_state: &FileSystemState) {
    loop {
        let mut command: String = String::new();
        eprint!("DIR2>");
        std::io::stdin().read_line(&mut command).unwrap();
        let command: String = command.trim().to_string();
        if command.is_empty() {
            continue;
        }
        let tokens = parse_command(&command);
        match tokens {
            Ok(command) => {
                // println!("{:?}", command);
                let res = execute_command(command);
                if res.unwrap().to_uppercase() == "exited!" {
                    break;
                }
            }
            Err(error) => {
                println!("Error: {}", error);
            }
        }
    }
}
