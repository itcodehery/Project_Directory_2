mod commands;
mod completion;
mod delegation;
mod favorites;
mod file_system_state;
mod filesystem;
mod docs;
mod indexer;
mod parser;
mod search;
mod sql_engine;
#[macro_use]
pub mod utils;
pub mod tui;
pub mod jobs;

use favorites::FavoritesManager;
use file_system_state::FileSystemState;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let mut current_file_sys_state: FileSystemState = FileSystemState::new();
    let mut fav_manager = FavoritesManager::new().expect("Failed to initialize FavoritesManager");
    
    // Execute .dir2rc
    if let Some(mut home_dir) = dirs::home_dir() {
        home_dir.push(".dir2rc");
        if home_dir.exists() {
            if let Ok(contents) = std::fs::read_to_string(home_dir) {
                for line in contents.lines() {
                    let cmd_line = line.trim();
                    if cmd_line.is_empty() || cmd_line.starts_with('#') {
                        continue;
                    }
                    let mut cmd = utils::substitute_env_vars(cmd_line);
                    cmd = current_file_sys_state.expand_aliases(&cmd);
                    if let Ok(command) = parser::parse_command(&cmd) {
                        let _ = commands::execute_command(command, &mut current_file_sys_state, &mut fav_manager).await;
                    }
                }
            }
        }
    }

    if let Err(e) = tui::run_tui(current_file_sys_state, fav_manager).await {
        eprintln!("Error in TUI: {}", e);
    }
}

pub fn trim_quotes(path: &PathBuf) -> PathBuf {
    let cleaned = path
        .to_string_lossy()
        .chars()
        .filter(|&c| c != '"')
        .collect::<String>();
    PathBuf::from(cleaned)
}
