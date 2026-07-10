use crate::indexer;
use colored::Colorize;
use std::path::{Path, PathBuf};

use crate::favorites::{Favorite, FavoritesManager};
use crate::file_system_state::FileSystemState;
use crate::filesystem;
use crate::filesystem::{get_directory_without_parent, get_file_metadata, is_dir};
use crate::parser::Command;
use crate::search::{SearchEngine, search_builder};
use rust_search::similarity_sort;

pub async fn execute_command(
    command: Command,
    file_system_state: &mut FileSystemState,
    favorites_manager: &mut FavoritesManager,
) -> Result<String, String> {
    match command {
        // Meta/System Commands
        Command::ListCommands => execute_list_all_cmd(),
        Command::ClearScreen => {
            crate::utils::set_clear_marker();
            return Ok(String::new());
        }
        Command::SqlQuery { query } => {
            return crate::sql_engine::execute_sql_query(file_system_state, &query);
        }
        Command::Docs { command_name } => {
            return crate::docs::show_docs(command_name);
        }
        Command::History => {
            crate::utils::reset_clear_marker();
            return Ok(String::new());
        }
        Command::Jobs | Command::Fg { .. } | Command::Kill { .. } => {
            // These are primarily intercepted and handled in tui.rs directly.
            // If they reach here (e.g., from .dir2rc), we just ignore them.
            return Ok(String::new());
        }
        Command::Config => {
            let mut home_dir = match dirs::home_dir() {
                Some(path) => path,
                None => return Err(String::from("Could not find home directory.")),
            };
            home_dir.push(".dir2rc");
            
            if !home_dir.exists() {
                let _ = std::fs::write(&home_dir, "# DIR2 Configuration File\n# Add aliases and exports here\n");
            }
            
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            
            let mut stdout = std::io::stdout();
            let _ = crossterm::terminal::disable_raw_mode();
            let _ = crossterm::execute!(stdout, crossterm::terminal::LeaveAlternateScreen, crossterm::event::DisableMouseCapture);
            
            let mut child = std::process::Command::new(&editor)
                .arg(home_dir.to_str().unwrap())
                .spawn();
                
            if let Ok(mut c) = child {
                let _ = c.wait();
            } else {
                crate::cprintln!("Failed to execute editor: {}", editor);
            }
            
            let _ = crossterm::terminal::enable_raw_mode();
            let _ = crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen, crossterm::event::EnableMouseCapture);
            return Ok(String::from("Closed config file. Restart dir2 to apply changes."));
        }
        Command::Exit => Ok("exited!".to_string()),
        Command::Export { key, value } => {
            if key.is_empty() || key.contains('=') || key.contains('\0') {
                return Err(format!("Invalid variable name: '{}'", key));
            }
            unsafe { std::env::set_var(&key, &value) };
            return Ok(format!("Exported {}={}", key, value));
        }
        Command::Unset { key } => {
            if key.is_empty() || key.contains('=') || key.contains('\0') {
                return Err(format!("Invalid variable name: '{}'", key));
            }
            unsafe { std::env::remove_var(&key) };
            return Ok(format!("Unset {}", key));
        }
        Command::Env => {
            for (key, value) in std::env::vars() {
                crate::cprintln!("{}={}", key, value);
            }
            return Ok(String::new());
        }
        Command::Echo { text } => {
            crate::cprintln!("{}", text);
            return Ok(String::new());
        }
        Command::Alias { key, value } => {
            file_system_state.aliases.insert(key.clone(), value.clone());
            return Ok(format!("Alias set: {}='{}'", key, value));
        }
        Command::Unalias { key } => {
            if file_system_state.aliases.remove(&key).is_some() {
                return Ok(format!("Alias removed: {}", key));
            } else {
                return Err(format!("Alias not found: {}", key));
            }
        }
        Command::Aliases => {
            if file_system_state.aliases.is_empty() {
                crate::cprintln!("No aliases defined.");
            } else {
                for (key, value) in &file_system_state.aliases {
                    crate::cprintln!("alias {}='{}'", key, value);
                }
            }
            return Ok(String::new());
        }
        Command::AddInteractive { command } => {
            let cmd_lower = command.to_lowercase();
            if !file_system_state.interactive_commands.contains(&cmd_lower) {
                file_system_state.interactive_commands.push(cmd_lower.clone());
            }
            return Ok(format!("Added {} to interactive commands list.", cmd_lower));
        }
        Command::RemoveInteractive { command } => {
            let cmd_lower = command.to_lowercase();
            if let Some(pos) = file_system_state.interactive_commands.iter().position(|x| *x == cmd_lower) {
                file_system_state.interactive_commands.remove(pos);
                return Ok(format!("Removed {} from interactive commands list.", cmd_lower));
            }
            return Err(format!("Command {} not found in interactive list.", cmd_lower));
        }
        Command::ListInteractive => {
            crate::cprintln!("Interactive Commands:");
            for cmd in &file_system_state.interactive_commands {
                crate::cprintln!("  - {}", cmd);
            }
            return Ok(String::new());
        }
        Command::Unknown { command, args } => {
            let cmd_lower = command.to_lowercase();
            let is_interactive = file_system_state.interactive_commands.contains(&cmd_lower);

            if is_interactive {
                let mut stdout = std::io::stdout();
                let _ = crossterm::terminal::disable_raw_mode();
                let _ = crossterm::execute!(stdout, crossterm::terminal::LeaveAlternateScreen, crossterm::event::DisableMouseCapture);

                let mut child = std::process::Command::new(&command)
                    .args(&args)
                    .current_dir(file_system_state.get_current_path())
                    .spawn();

                if let Ok(mut c) = child {
                    let _ = c.wait();
                } else {
                    crate::cprintln!("Failed to execute interactive command: {}", command);
                }

                let _ = crossterm::terminal::enable_raw_mode();
                let _ = crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen, crossterm::event::EnableMouseCapture);
                
                // Clear the screen fully by resetting the clear marker
                crate::utils::set_clear_marker();
                return Ok(String::new());
            }

            // attempt native execution for non-interactive with streaming output
            use std::process::Stdio;
            use tokio::io::{AsyncBufReadExt, BufReader};

            let mut child = tokio::process::Command::new(&command)
                .args(&args)
                .current_dir(file_system_state.get_current_path())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match child {
                Ok(mut c) => {
                    let stdout = c.stdout.take().unwrap();
                    let stderr = c.stderr.take().unwrap();

                    let mut stdout_reader = BufReader::new(stdout).lines();
                    let mut stderr_reader = BufReader::new(stderr).lines();

                    loop {
                        tokio::select! {
                            Ok(Some(line)) = stdout_reader.next_line() => {
                                crate::cprintln!("{}", line);
                            }
                            Ok(Some(line)) = stderr_reader.next_line() => {
                                crate::cprintln!("{}", line);
                            }
                            else => break,
                        }
                    }
                    
                    let _ = c.wait().await;
                    return Ok(String::new());
                }
                Err(_) => {
                    crate::cprintln!(
                        "Error: Unknown command '{}'.\nType {} to view a list of available commands.",
                        command,
                        "LC".yellow().to_string()
                    );
                    return Ok(String::new());
                }
            }
        }

        // Directory Navigation Commands
        Command::DodgeDirectory => execute_dodge_directory(file_system_state),
        Command::WatchDirectory {
            directory: _directory,
        } => execute_watch_directory(file_system_state, &_directory),
        Command::ListDirectory { show_hidden } => {
            return execute_list_directory(file_system_state, show_hidden);
        }
        Command::ChangeDrive { drive: _drive } => execute_change_drive(file_system_state, _drive),
        Command::MakeDirectory { directory } => {
            execute_make_directory(file_system_state, &directory)
        }
        Command::RemoveDirectory { directory } => {
            execute_remove_directory(file_system_state, &directory)
        }
        Command::RenameDirectory {
            old_directory,
            new_directory,
        } => execute_rename_directory(file_system_state, &old_directory, &new_directory),

        // File Management Commands
        Command::MakeFile { filename } => execute_make_file(file_system_state, &filename),
        Command::RemoveFile { filename } => execute_remove_file(file_system_state, &filename),
        Command::RenameFile {
            old_filename,
            new_filename,
        } => execute_rename_file(file_system_state, &old_filename, &new_filename),

        // State Management Commands
        Command::Select {
            filename: _filename,
            directory: _directory,
        } => execute_select(file_system_state, _filename, _directory),
        Command::ViewState => execute_view_state(file_system_state),
        Command::ClearState => execute_clear_state(file_system_state),
        Command::MetaState => execute_meta_state(file_system_state),
        Command::RunState => execute_run_state(file_system_state),

        // Search Commands
        Command::Search {
            engine,
            query: _query,
        } => execute_search(&engine, &_query),

        // Favorites Management Commands
        Command::FavView => execute_fav_view(favorites_manager),
        Command::FavRm { index: _index } => execute_remove_fav(_index, favorites_manager),
        Command::FavSet => execute_fav_set(file_system_state, favorites_manager),
        Command::RunFav { index: _index } => execute_run_fav(_index, favorites_manager),
    }
}
pub fn execute_change_drive(
    file_system_state: &mut FileSystemState,
    drive: String,
) -> Result<String, String> {
    // Validate drive letter format
    let drive_upper = drive.to_uppercase();

    // Check if it's a single letter
    if drive_upper.len() != 1 {
        return Err("Drive must be a single letter (A-Z)".to_string());
    }

    let drive_char = drive_upper.chars().next().unwrap();

    // Check if it's a valid letter
    if !drive_char.is_ascii_alphabetic() {
        return Err("Drive must be a letter (A-Z)".to_string());
    }

    // Format the drive path
    let drive_path = format!("{}:\\", drive_char);

    // Check if the drive exists
    let path = std::path::Path::new(&drive_path);
    if !path.exists() {
        return Err(format!(
            "Drive {} does not exist or is not accessible",
            drive_char
        ));
    }

    // Change to the drive
    match std::env::set_current_dir(&drive_path) {
        Ok(_) => {
            // Update your file system state's current directory
            file_system_state.set_current_directory(
                std::env::current_dir()
                    .map_err(|e| format!("Failed to get current directory: {}", e))?,
            );

            Ok(format!("Changed to drive {}", drive_char))
        }
        Err(e) => Err(format!("Failed to change to drive {}: {}", drive_char, e)),
    }
}

pub fn execute_list_all_cmd() -> Result<String, String> {
    let titles = [
        "DIR2 Commands (All Case-insensitive)",
        "---------------------",
        "Meta Commands:",
        "Environment Commands:",
        "Alias Commands:",
        "TUI Configuration:",
        "Directory/File Commands:",
        "State Commands:",
        "Favorites Commands:",
        "Search Commands:",
    ];

    let meta_commands = [
        ("CLS | /C | CLEAR", "Clear Screen"),
        ("ECHO <text>", "Prints text to the terminal"),
        ("DOCS <cmd>", "Shows the comprehensive manual for a command"),
        ("HISTORY | HIST", "Restores the log history after clearing the screen"),
        ("CONFIG | RC", "Opens ~/.dir2rc in your default $EDITOR"),
        ("LC", "Lists Commands"),
        ("WD", "Watch Directory"),
        ("LD", "List Directory"),
        ("DD", "Dodge Directory"),
        ("CD", "Change Drive"),
        ("EXIT | /E", "Exit Terminal"),
    ];

    let env_commands = [
        ("EXPORT <VAR>=<value>", "Sets an environment variable"),
        ("UNSET <VAR>", "Removes an environment variable"),
        ("ENV", "Lists all environment variables"),
    ];

    let alias_commands = [
        ("ALIAS <name>='<cmd>'", "Sets a command alias"),
        ("UNALIAS <name>", "Removes an alias"),
        ("ALIASES", "Lists all aliases"),
    ];

    let tui_commands = [
        ("TUIADD <command>", "Adds a command to the interactive whitelist"),
        ("TUIRM <command>", "Removes a command from the whitelist"),
        ("TUILS", "Lists all interactive whitelist commands"),
    ];

    let dir_file_commands = [
        ("MKDIR <directory>", "Creates a directory"),
        ("RMDIR <directory>", "Removes a directory"),
        (
            "RENDIR <old_directory> <new_directory>",
            "Renames a directory",
        ),
        ("MKFILE <filename>", "Creates a file"),
        ("RMFILE <filename>", "Removes a file"),
        ("RENFILE <old_filename> <new_filename>", "Renames a file"),
    ];

    let state_commands = [
        (
            "SELECT <filename.ext> FROM <directory>",
            "Sets <filename.ext> file as current STATE",
        ),
        ("VIEW STATE | VS", "To view current STATE"),
        ("DROP STATE | DS", "Drops the current STATE"),
        ("META STATE | MS", "To view current STATE File Metadata"),
        (
            "RUN STATE | RS",
            "Runs the file or script present in the current STATE",
        ),
    ];

    let search_commands = [
        ("S / SEARCH <engine> <query>", "Searches using specified engine"),
    ];
    let fav_commands = [
        ("FAV VIEW", "View all Favorites as a List"),
        ("FAV RM <index>", "Removes <filename> from favorites"),
        ("FAV SET STATE", "Sets current state as latest favorite"),
        (
            "RUN FAV <index>",
            "Runs the file at the index of the Favorites list",
        ),
    ];

    crate::cprintln!("\n{}\n{}", titles[0], titles[1]);

    crate::cprintln!("{}", titles[2]);
    for (command, description) in meta_commands.iter() {
        crate::cprintln!("{} : {}", command.bright_blue(), description);
    }

    crate::cprintln!("\n{}", titles[3]);
    for (command, description) in env_commands.iter() {
        crate::cprintln!("{} : {}", command.magenta(), description);
    }

    crate::cprintln!("\n{}", titles[4]);
    for (command, description) in alias_commands.iter() {
        crate::cprintln!("{} : {}", command.bright_green(), description);
    }

    crate::cprintln!("\n{}", titles[5]);
    for (command, description) in tui_commands.iter() {
        crate::cprintln!("{} : {}", command.blue(), description);
    }

    crate::cprintln!("\n{}", titles[6]);
    for (command, description) in dir_file_commands.iter() {
        crate::cprintln!("{} : {}", command.bright_cyan(), description);
    }

    crate::cprintln!("\n{}", titles[7]);
    for (command, description) in state_commands.iter() {
        crate::cprintln!("{} : {}", command.yellow(), description);
    }

    crate::cprintln!("\n{}", titles[8]);
    for (command, description) in fav_commands.iter() {
        crate::cprintln!("{} : {}", command.green(), description);
    }

    crate::cprintln!("\n{}", titles[9]);
    for (command, description) in search_commands.iter() {
        crate::cprintln!("{} : {}", command.bright_magenta(), description);
    }

    crate::cprintln!();

    Ok("Executed: List Commands".to_string())
}
pub fn execute_dodge_directory(sys_state: &mut FileSystemState) -> Result<String, String> {
    let current_path = sys_state.get_current_path();

    // Check if the path is a directory
    if !is_dir(current_path) {
        return Ok(String::from("Failed: Not a directory"));
    }

    // Safely get parent directory
    match current_path.parent() {
        Some(parent) => {
            sys_state.set_current_directory(parent.to_path_buf());
            Ok(String::from("Executed: Dodge Directory"))
        }
        None => {
            crate::cprintln!(
                "Error: {} If multiple drives exist, use CD to Switch Drives.",
                "Cannot dodge Root Directory!".red().to_string()
            );
            Ok(String::from("Failed: Dodge Directory"))
        }
    }
    // Err(String::from("Cannot navigate to parent directory"))
}

pub fn execute_select(
    sys_state: &mut FileSystemState,
    filename: String,
    directory: String,
) -> Result<String, String> {
    let dir_path: PathBuf = if directory.is_empty() || directory == "/" {
        sys_state.get_current_path().clone()
    } else {
        let path = Path::new(&directory);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            filesystem::resolve_path(path, sys_state.get_current_path())
        }
    };

    if !filesystem::path_exists(&dir_path) {
        crate::cprintln!("Directory {} does not exist!", directory.bright_red());
        return Ok(format!("Directory {} does not exist!", directory));
    }
    if !filesystem::is_dir(&dir_path) {
        crate::cprintln!("{} is not a directory!", directory.bright_red());
        return Ok(format!("Directory {} is not a directory!", directory));
    }

    sys_state.set_current_directory(dir_path.clone());

    let file_path = dir_path.join(&filename);

    if !filesystem::path_exists(&file_path) {
        crate::cprintln!(
            "File {} does not exist in Directory!",
            file_path.display().to_string().bright_red()
        );
        return Ok(format!(
            "File {} does not exist in Directory {}!",
            file_path.display(),
            directory
        ));
    }
    if !filesystem::is_file(&file_path) {
        crate::cprintln!(
            "{} is not a file!",
            file_path.display().to_string().bright_red()
        );
        return Ok(format!("File {} is not a file!", file_path.display()));
    }
    sys_state.set_current_state(file_path.clone());

    crate::cprintln!(
        "Selected STATE: {}",
        file_path.display().to_string().green()
    );
    return Ok(format!("Selected {}", file_path.display()));
}
pub fn execute_run_state(sys_state: &mut FileSystemState) -> Result<String, String> {
    let file_path = match sys_state.get_current_state() {
        Some(path) => path,
        None => {
            crate::cprintln!(
                "\nError: {}",
                "No file selected. Use SELECT command first"
                    .red()
                    .to_string()
            );
            return Ok(String::from("No file selected. Use SELECT command first."));
        }
    };
    execute_file(file_path).expect("Couldn't run file!");
    return Ok(String::from("Running"));
}
pub fn execute_file(file_path: &PathBuf) -> Result<String, String> {
    // Validate file still exists and is executable
    if !filesystem::path_exists(file_path) {
        crate::cprintln!(
            "\nError: {}",
            "Selected file no longer exists".red().to_string()
        );
        return Ok(format!(
            "Error: Selected file '{}' no longer exists",
            file_path.display().to_string().red().to_string()
        ));
    }

    if !filesystem::is_executable(file_path) {
        // For non-executable files, open with default application
        match open::that(file_path) {
            Ok(_) => {
                crate::cprintln!(
                    "Opening STATE with default application: {}",
                    file_path.display().to_string().green()
                );
                Ok(format!(
                    "Opened '{}' with default application",
                    file_path.display()
                ))
            }
            Err(e) => {
                crate::cprintln!(
                    "\nError: {} -> {}",
                    "Failed to open STATE".red().to_string(),
                    file_path.display().to_string().red()
                );
                Ok(format!(
                    "Failed to open STATE: '{}': {}",
                    file_path.display().to_string().red(),
                    e
                ))
            }
        }
    } else {
        match std::process::Command::new(file_path).spawn() {
            Ok(_child) => {
                crate::cprintln!("Running STATE: {}", file_path.display().to_string().green());
                Ok(format!("Started: {}", file_path.display()))
            }
            Err(e) => {
                crate::cprintln!(
                    "\nError: {} -> {}",
                    "Failed to run STATE".red().to_string(),
                    e.to_string().red().to_string()
                );
                Ok(format!(
                    "Failed to run STATE: {}",
                    e.to_string().red().to_string()
                ))
            }
        }
    }
}

pub fn execute_view_state(sys_state: &mut FileSystemState) -> Result<String, String> {
    let current_state = sys_state.get_current_state();
    if current_state.is_none() {
        crate::cprintln!("{}:\nState: None", "Current STATE".yellow());
    } else {
        crate::cprintln!(
            "{}:\nState: {}",
            "Current STATE".yellow(),
            current_state.clone().unwrap().to_str().unwrap()
        );
    }
    return Ok(String::from("Executed View State!"));
}

pub fn execute_clear_state(sys_state: &mut FileSystemState) -> Result<String, String> {
    sys_state.clear_state();
    crate::cprintln!("{}", "STATE Dropped".yellow());
    return Ok(String::from("Executed Clear State!"));
}

pub fn execute_watch_directory(
    sys_state: &mut FileSystemState,
    directory: &String,
) -> Result<String, String> {
    // crate::cprintln!("DEBUG: Looking for directory: '{}'", directory);
    // crate::cprintln!("DEBUG: Current path: '{}'", sys_state.get_current_path().display());
    let dir_path = PathBuf::from(directory);
    // crate::cprintln!("DEBUG: Full path to check: '{}'", dir_path.display());
    // crate::cprintln!("DEBUG: Current Directory according to the system: {} ", std::env::current_dir().unwrap().to_str().unwrap());
    // crate::cprintln!("DEBUG: Path exists: {}", dir_path.exists());
    // crate::cprintln!("DEBUG: Is directory: {}", dir_path.is_dir());
    // Check if directory exists
    if !filesystem::path_exists(&dir_path) {
        crate::cprintln!(
            "Error: Directory {} does not exist!",
            directory.red().to_string()
        );
        return Ok(format!("Directory '{}' does not exist", directory));
    }

    // Check if it's actually a directory
    if !filesystem::is_dir(&dir_path) {
        crate::cprintln!("Error: {} is not a directory!", directory.red().to_string());
        return Ok(format!("'{}' is not a directory", directory));
    }

    // Update current directory
    sys_state.set_current_directory(sys_state.get_current_path().join(dir_path));

    // Index current directory
    indexer::index_current_directory(sys_state);

    // Clear index
    sys_state.clear_index();

    Ok(format!("Changed to directory: {}", directory))
}

pub fn execute_list_directory(sys_state: &mut FileSystemState, show_hidden: bool) -> Result<String, String> {
    let current_path = sys_state.get_current_path();

    if !is_dir(current_path) {
        return Err(String::from("Not a Directory"));
    } else {
        use comfy_table::{Table, Cell, Color as CColor, Attribute};
        use comfy_table::presets::UTF8_FULL;
        use chrono::{DateTime, Local};
        use std::time::SystemTime;

        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec![
            Cell::new("Name").fg(CColor::Green).add_attribute(Attribute::Bold),
            Cell::new("Type").fg(CColor::Green).add_attribute(Attribute::Bold),
            Cell::new("Size").fg(CColor::Green).add_attribute(Attribute::Bold),
            Cell::new("Modified").fg(CColor::Green).add_attribute(Attribute::Bold),
        ]);

        for entry in current_path
            .read_dir()
            .expect("read_dir call failed")
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            
            if !show_hidden && name.starts_with('.') {
                continue;
            }
            
            let metadata = entry.metadata().ok();
            
            let is_dir = path.is_dir();
            let file_type = if is_dir { "Dir" } else { "File" };
            
            let name_color = if is_dir { CColor::Green } else { CColor::White };
            let type_color = if is_dir { CColor::Green } else { CColor::DarkGrey };
            
            let size = if let Some(m) = &metadata {
                if is_dir {
                    "-".to_string()
                } else {
                    format!("{} B", m.len())
                }
            } else {
                "-".to_string()
            };

            let modified = if let Some(m) = &metadata {
                if let Ok(sys_time) = m.modified() {
                    let datetime: DateTime<Local> = sys_time.into();
                    datetime.format("%Y-%m-%d %H:%M").to_string()
                } else {
                    "-".to_string()
                }
            } else {
                "-".to_string()
            };

            table.add_row(vec![
                Cell::new(name).fg(name_color),
                Cell::new(file_type).fg(type_color),
                Cell::new(size).fg(CColor::White),
                Cell::new(modified).fg(CColor::DarkGrey),
            ]);
        }
        
        crate::cprintln!("\n{}", table);
    }
    return Ok(String::from("Executed: List Directory"));
}

fn execute_make_directory(
    sys_state: &mut FileSystemState,
    directory: &String,
) -> Result<String, String> {
    let current_path = sys_state.get_current_path();
    let new_path = current_path.join(directory);
    if filesystem::path_exists(&new_path) {
        return Err(format!("Directory '{}' already exists", directory));
    }
    if filesystem::is_dir(&new_path) {
        return Err(format!("'{}' is already a directory", directory));
    }
    if filesystem::create_dir(&new_path) {
        sys_state.set_current_directory(new_path);
        return Ok(format!("Created directory '{}'", directory));
    }
    return Err(format!("Failed to create directory '{}'", directory));
}

fn execute_remove_directory(
    sys_state: &mut FileSystemState,
    directory: &String,
) -> Result<String, String> {
    let current_path = sys_state.get_current_path();
    let new_path = current_path.join(directory);
    if filesystem::is_dir(&new_path) {
        if filesystem::remove_dir(&new_path) {
            return Ok(format!("Removed directory '{}'", directory));
        } else {
            return Err(format!("Failed to remove directory '{}'", directory));
        }
    }
    return Err(format!("Failed to create directory '{}'", directory));
}

fn execute_rename_directory(
    sys_state: &mut FileSystemState,
    directory: &String,
    new_name: &String,
) -> Result<String, String> {
    let current_path = sys_state.get_current_path();
    let new_path = current_path.join(directory);
    if filesystem::path_exists(&new_path) {
        if filesystem::is_dir(&new_path) {
            if filesystem::rename(&new_path, &current_path.join(new_name)) {
                return Ok(format!(
                    "Renamed directory '{}' to '{}'",
                    directory, new_name
                ));
            } else {
                return Err(format!(
                    "Failed to rename directory '{}' to '{}'",
                    directory, new_name
                ));
            }
        }
    }
    return Err(format!("Failed to create directory '{}'", directory));
}

fn execute_make_file(sys_state: &mut FileSystemState, file: &String) -> Result<String, String> {
    let current_path = sys_state.get_current_path();
    let new_path = current_path.join(file);
    if filesystem::path_exists(&new_path) {
        return Err(format!("File '{}' already exists", file));
    }
    if filesystem::create_file(&new_path) {
        return Ok(format!("Created file '{}'", file));
    }
    return Err(format!("Failed to create file '{}'", file));
}

fn execute_remove_file(sys_state: &mut FileSystemState, file: &String) -> Result<String, String> {
    let current_path = sys_state.get_current_path();
    let new_path = current_path.join(file);
    if filesystem::path_exists(&new_path) {
        if filesystem::remove_file(&new_path) {
            return Ok(format!("Removed file '{}'", file));
        } else {
            return Err(format!("Failed to remove file '{}'", file));
        }
    }
    return Err(format!("Failed to remove file '{}'", file));
}

fn execute_rename_file(
    sys_state: &mut FileSystemState,
    file: &String,
    new_name: &String,
) -> Result<String, String> {
    let current_path = sys_state.get_current_path();
    let new_path = current_path.join(file);
    if filesystem::path_exists(&new_path) {
        return Err(format!("File '{}' already exists", file));
    }
    if filesystem::rename(&new_path, &current_path.join(new_name)) {
        return Ok(format!("Renamed file '{}' to '{}'", file, new_name));
    } else {
        return Err(format!(
            "Failed to rename file '{}' to '{}'",
            file, new_name
        ));
    }
}
pub fn execute_meta_state(sys_state: &mut FileSystemState) -> Result<String, String> {
    let current_state = sys_state.get_current_state();

    if current_state.is_none() {
        crate::cprintln!("Error: STATE is Empty!");
        return Ok(String::from("STATE is Empty!"));
    } else {
        let metadata = get_file_metadata(current_state.as_ref().unwrap())
            .expect("ERROR > Failed to get metadata");
        crate::cprintln!(
            "\nCurrent STATE: {}",
            current_state.clone().unwrap().to_str().unwrap()
        );
        crate::cprintln!("\n{}", "STATE Metadata:".yellow());
        crate::cprintln!(
            "File Name: {}",
            current_state.clone().unwrap().to_str().unwrap()
        );
        crate::cprintln!("File Size: {}", metadata.size.to_string());
        crate::cprintln!("Last Modified: {:?}", metadata.modified.unwrap());
        crate::cprintln!("Read Only: {}\n", metadata.is_readonly.to_string());
        return Ok(String::from("Executed: STATE Metadata"));
    }
}



pub fn execute_search(engine: &str, query: &str) -> Result<String, String> {
    return match engine.to_uppercase().as_str() {
        "GOOGLE" | "G" => {
            crate::cprintln!(
                "\n{}: Searching using Google...",
                query.yellow()
            );
            open::that(format!(
                "https://www.google.com/search?q={}",
                query
            ))
            .expect("Couldn't launch Google!");
            Ok(format!("Opened '{}' with Google", query))
        }
        "DDG" | "D" => {
            crate::cprintln!(
                "\n{}: Searching using DuckDuckGo...",
                query.yellow()
            );
            open::that(format!(
                "https://duckduckgo.com/?t=ffab&q={}",
                query
            ))
            .expect("Couldn't launch DuckDuckGo!");
            Ok(format!("Opened '{}' with DuckDuckGo", query))
        }
        "PERPLEXITY" | "P" => {
            crate::cprintln!(
                "\n{}: Searching using Perplexity...",
                query.yellow()
            );
            open::that(format!(
                "https://www.perplexity.ai/search?q={}",
                query
            ))
            .expect("Couldn't launch Perplexity!");
            Ok(format!("Opened '{}' with Perplexity", query))
        }
        "CHATGPT" | "C" => {
            crate::cprintln!(
                "\n{}: Searching using ChatGPT...",
                query.yellow()
            );
            open::that(format!(
                "https://chatgpt.com/?q={}",
                query
            ))
            .expect("Couldn't launch ChatGPT!");
            Ok(format!("Opened '{}' with ChatGPT", query))
        }
        "CLAUDE" | "CL" => {
            crate::cprintln!(
                "\n{}: Searching using Claude...",
                query.yellow()
            );
            open::that(format!(
                "https://claude.ai/new?q={}",
                query
            ))
            .expect("Couldn't launch Claude!");
            Ok(format!("Opened '{}' with Claude", query))
        }
        "GEMINI" | "GM" => {
            crate::cprintln!(
                "\n{}: Searching using Gemini...",
                query.yellow()
            );
            open::that(format!(
                "https://gemini.google.com/app?q={}",
                query
            ))
            .expect("Couldn't launch Gemini!");
            Ok(format!("Opened '{}' with Gemini", query))
        }
        _ => Err(String::from("Unsupported search engine. Use Google, DDG, ChatGPT, Perplexity, Claude, or Gemini.")),
    };
}

// ------------------------------------------------------
// ---------------------FAVORITES------------------------
// ------------------------------------------------------

pub fn execute_fav_view(favorites_manager: &mut FavoritesManager) -> Result<String, String> {
    let states = favorites_manager.get_all();
    if states.is_empty() {
        crate::cprintln!(
            "No favorites found! Add a favorite by using {}",
            "FAV SET STATE".yellow()
        );
        return Ok(String::from(
            "No matches found! Try switching root directories.",
        ));
    }
    crate::cprintln!("\nFavorites List:");
    for (index, state) in states.iter().enumerate() {
        crate::cprintln!(
            "{}: {} > {} ",
            index.to_string().bright_blue(),
            state.get_alias_name().yellow(),
            state.get_path().to_string_lossy()
        )
    }
    crate::cprintln!(
        "\nUse {} to run Favorite at index.",
        "RUN FAV <index> or RF <index>".yellow()
    );
    return Ok(String::from("Done!"));
}

pub fn execute_fav_set(
    file_system_state: &mut FileSystemState,
    favorites_manager: &mut FavoritesManager,
) -> Result<String, String> {
    let current_state: PathBuf;
    match file_system_state.get_current_state().clone() {
        Some(state) => {
            current_state = state;
        }
        None => {
            crate::cprintln!(
                "ERROR: {}",
                "Couldn't get current STATE. STATE might be empty.".red()
            );
            return Ok(String::from(""));
        }
    };
    let favs = favorites_manager.get_all();
    if favs.is_empty() {
        let new_fav = Favorite::from(current_state.clone());
        favorites_manager
            .add(new_fav)
            .expect("Couldn't add favorites to Favorites Manager!");
        crate::cprintln!(
            "Added STATE {} to Favorites. Use FAV VIEW to view Favorites list.",
            current_state.display().to_string().green()
        );
        return Ok(String::from("Completed FAV SET"));
    }
    if favs.len() + 1 > 10 {
        crate::cprintln!("ERROR: Favorites List is full! (Max Favorites = 10)");
        return Ok(String::from("FAV SET TOO_MANY!"));
    } else {
        let new_fav = Favorite::from(current_state.clone());
        favorites_manager
            .add(new_fav)
            .expect("Couldn't add favorites to Favorite Manager!");
        crate::cprintln!(
            "Added STATE {} to Favorites Manager!",
            current_state.display().to_string().green()
        );
        return Ok(String::from("Completed FAV SET"));
    }
}

fn execute_run_fav(
    index: usize,
    favorites_manager: &mut FavoritesManager,
) -> Result<String, String> {
    let fav = favorites_manager
        .get_by_index(index)
        .expect("Couldn't get favorite");

    execute_file(fav.get_path())
}

fn execute_remove_fav(
    index: usize,
    favorites_manager: &mut FavoritesManager,
) -> Result<String, String> {
    if favorites_manager.is_empty() || index >= favorites_manager.len() {
        crate::cprintln!("ERROR: Index out of bounds!");
        return Ok(String::from("Invalid index!"));
    }
    match favorites_manager.remove(index) {
        Ok(_) => crate::cprintln!("Removed favorite from FavoritesManager!"),
        Err(msg) => {
            return Err(format!(
                "Failed to remove favorite from FavoritesManager: {}",
                msg
            ));
        }
    };

    return Ok("Removed favorites from Favorite Manager!".to_string());
}
