use std::path::PathBuf;
mod commands;
mod completion;
mod config;
mod favorites;
mod file_system_state;
mod filesystem;
mod indexing;
mod parser;
mod search;

use crate::commands::execute_command;
use crate::completion::Dir2Helper;
use colored::Colorize;
use favorites::FavoritesManager;
use file_system_state::FileSystemState;
use parser::parse_command;
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, Editor};
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
    println!(
        "{} for Windows\nInstall the latest DIR2 for new features and improvements!",
        "DIR2".green()
    );
    println!("------------------------");
    println!("Current State: {:?}", sys_state.get_current_state());
    // println!("Current Directory: {}\n", sys_state.get_current_path().to_string_lossy());
}

fn command_handler(sys_state: &mut FileSystemState, favorites_manager: &mut FavoritesManager) {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .auto_add_history(true)
        .build();
        
    let mut rl = Editor::with_config(config).expect("Failed to create readline editor");
    let mut helper = Dir2Helper::new();
    helper.update_current_directory(sys_state.get_current_path().clone());
    rl.set_helper(Some(helper));
    
    // Load command history if it exists
    let _ = rl.load_history("dir2_history.txt");
    
    loop {
        // Refresh the file index periodically
        if let Some(helper) = rl.helper_mut() {
            helper.refresh_index_if_needed();
        }
        
        let prompt = format!(
            "DIR2@{}> ",
            trim_quotes(sys_state.get_current_path()).to_string_lossy()
        );
        
        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let command = line.trim().to_string();
                if command.is_empty() {
                    continue;
                }
                
                // Add command to history
                rl.add_history_entry(command.as_str()).ok();
                
                let tokens = parse_command(&command);
                match tokens {
                    Ok(command) => {
                        // Handle IndexStats command specially since it needs access to the completion helper
                        if matches!(command, crate::parser::Command::IndexStats) {
                            if let Some(helper) = rl.helper() {
                                let (file_count, dir_count) = helper.get_index_stats();
                                println!("📁 {} Index Statistics:", "FILE".green());
                                println!("   📄 Files indexed: {}", file_count.to_string().cyan());
                                println!("   📂 Directories indexed: {}", dir_count.to_string().cyan());
                                println!("   📊 Total entries: {}", (file_count + dir_count).to_string().yellow());
                                println!("   🔍 Search scope: Global (all subdirectories)");
                            } else {
                                println!("❌ Index not available");
                            }
                        } else if matches!(command, crate::parser::Command::ListDirs) {
                            if let Some(helper) = rl.helper() {
                                println!("📂 Available directories in current path:");
                                let dirs = helper.get_directories_in_current_path("");
                                for (i, dir) in dirs.iter().enumerate() {
                                    println!("   {}. {}", i + 1, dir.cyan());
                                }
                                if dirs.is_empty() {
                                    println!("   No directories found");
                                }
                            } else {
                                println!("❌ Helper not available");
                            }
                        } else {
                            let res = execute_command(command, sys_state, favorites_manager);
                            
                            // Update the completion helper with the new current directory
                            if let Some(helper) = rl.helper_mut() {
                                helper.update_current_directory(sys_state.get_current_path().clone());
                            }
                            
                            if res.as_ref().map(|r| r.to_uppercase()) == Ok("EXITED!".to_string()) {
                                break;
                            }
                        }
                    }
                    Err(error) => {
                        println!("Error: {}", error);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    
    // Save command history
    let _ = rl.save_history("dir2_history.txt");
}

fn trim_quotes(path: &PathBuf) -> PathBuf {
    let cleaned = path
        .to_string_lossy()
        .chars()
        .filter(|&c| c != '"')
        .collect::<String>();
    PathBuf::from(cleaned)
}
