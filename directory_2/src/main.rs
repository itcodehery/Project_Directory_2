use std::path::PathBuf;

mod parser;
mod file_system_state;
mod filesystem;
mod indexing;
mod commands;
mod config;
mod search;
mod favorites;

use colored::Colorize;
use parser::parse_command;
use file_system_state::FileSystemState;
use crate::commands::execute_command;
use favorites::FavoritesManager;
fn main() {
    // Dependency Injection of the State Variable
    let mut current_file_sys_state: FileSystemState = FileSystemState::new();
    let mut fav_manager = FavoritesManager::new().expect("Failed to initialize FavoritesManager");
    println!("\x1B[2J\x1B[1;1H");
    terminal_boilerplate(&current_file_sys_state);
    command_handler(&mut current_file_sys_state, &mut fav_manager);
}

fn terminal_boilerplate(sys_state: &FileSystemState) {
    println!("------------------------");
    println!("{} for Windows\nInstall the latest DIR2 for new features and improvements!", "DIR2".green());
    println!("------------------------");
    println!("Current State: {:?}", sys_state.get_current_state());
    println!("Current Directory: {}\n", sys_state.get_current_path().to_string_lossy());
}

fn command_handler(sys_state: &mut FileSystemState, favorites_manager: &mut FavoritesManager) {
    loop {
        let mut command: String = String::new();
        eprint!("{}{}>","DIR2@".green(),trim_quotes(sys_state.get_current_path()).to_string_lossy());
        std::io::stdin().read_line(&mut command).unwrap();
        let command: String = command.trim().to_string();
        if command.is_empty() {
            continue;
        }
        let tokens = parse_command(&command);
        match tokens {
            Ok(command) => {
                let res = execute_command(command, sys_state, favorites_manager);
                if res.unwrap().to_uppercase() == "EXITED!" {
                    break;
                }

            }
            Err(error) => {
                println!("Error: {}", error);
            }
        }
    }
}

fn trim_quotes(path: &PathBuf) -> PathBuf {
    let cleaned = path.to_string_lossy()
        .chars()
        .filter(|&c| c != '"')
        .collect::<String>();
    PathBuf::from(cleaned)
}
