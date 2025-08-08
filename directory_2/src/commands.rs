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

pub fn execute_command(
    command: Command,
    file_system_state: &mut FileSystemState,
    favorites_manager: &mut FavoritesManager,
) -> Result<String, String> {
    match command {
        // Meta/System Commands
        Command::ListCommands => execute_list_all_cmd(),
        Command::ClearScreen => {
            // Windows-specific
            std::process::Command::new("cmd")
                .args(["/c", "cls"])
                .status()
                .ok();
            return Ok(String::from("Executed: Clear Screen"));
        }
        Command::Exit => Ok("exited!".to_string()),
        Command::Unknown { command } => {
            return Ok(String::from(format!(
                "Unexecuted: Unknown command {}",
                command
            )));
        }

        // Directory Navigation Commands
        Command::DodgeDirectory => execute_dodge_directory(file_system_state),
        Command::WatchDirectory {
            directory: _directory,
        } => execute_watch_directory(file_system_state, &_directory),
        Command::ListDirectory => execute_list_directory(file_system_state),
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
        Command::FindExact {
            filename: _filename,
        } => {
            println!(
                "FE > System-wide Search: Searching for {}...",
                _filename.to_string().yellow()
            );
            execute_find_exact(&_filename)
        }
        Command::Search {
            engine,
            filename: _filename,
        } => execute_search(&engine, &_filename),

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
        "Directory/File Commands:",
        "State Commands:",
        "Favorites Commands:",
        "Search Commands:",
    ];

    let meta_commands = [
        ("CLS | /C", "Clear Screen"),
        ("CML <command>", "Executes a command in the terminal"),
        ("LC", "Lists Commands"),
        ("WD", "Watch Directory"),
        ("LD", "List Directory"),
        ("DD", "Dodge Directory"),
        ("CD", "Change Drive"),
        ("EXIT | /E", "Exit Terminal"),
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
        (
            "FIND EXACT <query> | FE <query>",
            "Performs a System-wide File search on the Query, returns the list of Directories.",
        ),
        (
            "SEARCH GOOGLE <query> | S G <query>",
            "Performs a Web Query using Google as the search engine.",
        ),
        (
            "SEARCH DDG <query> | S D <query>",
            "Performs a Web Query using DuckDuckGo as the search engine.",
        ),
        (
            "SEARCH CHATGPT <query> | S C <query>",
            "Performs a query to ChatGPT using the query.",
        ),
        (
            "SEARCH PERPLEXITY <query> | S P <query>",
            "Performs a query to Perplexity using the query.",
        ),
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

    println!("\n{}\n{}", titles[0], titles[1]);

    println!("{}", titles[2]);
    for (command, description) in meta_commands.iter() {
        println!("{} : {}", command.bright_blue(), description);
    }

    println!("\n{}", titles[3]);
    for (command, description) in dir_file_commands.iter() {
        println!("{} : {}", command.bright_cyan(), description);
    }

    println!("\n{}", titles[4]);
    for (command, description) in state_commands.iter() {
        println!("{} : {}", command.yellow(), description);
    }

    println!("\n{}", titles[5]);
    for (command, description) in fav_commands.iter() {
        println!("{} : {}", command.green(), description);
    }

    println!("\n{}", titles[6]);
    for (command, description) in search_commands.iter() {
        println!("{} : {}", command.bright_purple(), description);
    }

    println!();

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
            println!(
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
        println!("Directory {} does not exist!", directory.bright_red());
        return Ok(format!("Directory {} does not exist!", directory));
    }
    if !filesystem::is_dir(&dir_path) {
        println!("{} is not a directory!", directory.bright_red());
        return Ok(format!("Directory {} is not a directory!", directory));
    }

    sys_state.set_current_directory(dir_path.clone());

    let file_path = dir_path.join(&filename);

    if !filesystem::path_exists(&file_path) {
        println!(
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
        println!(
            "{} is not a file!",
            file_path.display().to_string().bright_red()
        );
        return Ok(format!("File {} is not a file!", file_path.display()));
    }
    sys_state.set_current_state(file_path.clone());

    println!(
        "Selected STATE: {}",
        file_path.display().to_string().green()
    );
    return Ok(format!("Selected {}", file_path.display()));
}
pub fn execute_run_state(sys_state: &mut FileSystemState) -> Result<String, String> {
    let file_path = match sys_state.get_current_state() {
        Some(path) => path,
        None => {
            println!(
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
        println!(
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
        match std::process::Command::new("cmd")
            .args(["/c", "start", "", &file_path.to_string_lossy()])
            .spawn()
        {
            Ok(_child) => {
                println!(
                    "Opening STATE with default application: {}",
                    file_path.display().to_string().green()
                );
                Ok(format!(
                    "Opened '{}' with default application",
                    file_path.display()
                ))
            }
            Err(e) => {
                println!(
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
                println!("Running STATE: {}", file_path.display().to_string().green());
                Ok(format!("Started: {}", file_path.display()))
            }
            Err(e) => {
                println!(
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
        println!("{}:\nState: None", "Current STATE".yellow());
    } else {
        println!(
            "{}:\nState: {}",
            "Current STATE".yellow(),
            current_state.clone().unwrap().to_str().unwrap()
        );
    }
    return Ok(String::from("Executed View State!"));
}

pub fn execute_clear_state(sys_state: &mut FileSystemState) -> Result<String, String> {
    sys_state.clear_state();
    println!("{}", "STATE Dropped".yellow());
    return Ok(String::from("Executed Clear State!"));
}

pub fn execute_watch_directory(
    sys_state: &mut FileSystemState,
    directory: &String,
) -> Result<String, String> {
    // println!("DEBUG: Looking for directory: '{}'", directory);
    // println!("DEBUG: Current path: '{}'", sys_state.get_current_path().display());
    let dir_path = PathBuf::from(directory);
    // println!("DEBUG: Full path to check: '{}'", dir_path.display());
    // println!("DEBUG: Current Directory according to the system: {} ", std::env::current_dir().unwrap().to_str().unwrap());
    // println!("DEBUG: Path exists: {}", dir_path.exists());
    // println!("DEBUG: Is directory: {}", dir_path.is_dir());
    // Check if directory exists
    if !filesystem::path_exists(&dir_path) {
        println!(
            "Error: Directory {} does not exist!",
            directory.red().to_string()
        );
        return Ok(format!("Directory '{}' does not exist", directory));
    }

    // Check if it's actually a directory
    if !filesystem::is_dir(&dir_path) {
        println!("Error: {} is not a directory!", directory.red().to_string());
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

pub fn execute_list_directory(sys_state: &mut FileSystemState) -> Result<String, String> {
    let current_path = sys_state.get_current_path();

    if !is_dir(current_path) {
        return Err(String::from("Not a Directory"));
    } else {
        println!(
            "Contents of current directory: {}",
            current_path.to_str().unwrap().yellow()
        );
        for (index, entry) in current_path
            .read_dir()
            .expect("read_dir call failed")
            .enumerate()
        {
            if let Ok(entry) = entry {
                println!(
                    "{}\t> {}",
                    index.to_string().bright_blue(),
                    get_directory_without_parent(&*entry.path())
                );
            }
        }
        println!("\n");
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
        println!("Error: STATE is Empty!");
        return Ok(String::from("STATE is Empty!"));
    } else {
        let metadata = get_file_metadata(current_state.as_ref().unwrap())
            .expect("ERROR > Failed to get metadata");
        println!(
            "\nCurrent STATE: {}",
            current_state.clone().unwrap().to_str().unwrap()
        );
        println!("\n{}", "STATE Metadata:".yellow());
        println!(
            "File Name: {}",
            current_state.clone().unwrap().to_str().unwrap()
        );
        println!("File Size: {}", metadata.size.to_string());
        println!("Last Modified: {:?}", metadata.modified.unwrap());
        println!("Read Only: {}\n", metadata.is_readonly.to_string());
        return Ok(String::from("Executed: STATE Metadata"));
    }
}

pub fn execute_find_exact(query: &String) -> Result<String, String> {
    // let current_state = sys_state.get_current_state();

    let mut search = search_builder(query);
    if !search.is_empty() {
        println!(
            "\n{}: Found '{}' at these directories:",
            "FIND EXACT".yellow(),
            query.yellow().to_string()
        );
        similarity_sort(&mut search, &query);
        for (index, str) in search.iter().enumerate() {
            println!("\n{}> {}", (index + 1).to_string().bright_blue(), str);
        }
        println!("\n");
    } else {
        println!(
            "\n{}: No matches found! Try switching root directories.",
            "FIND EXACT".yellow()
        );
    }

    return Ok(String::from("Finished search!"));
}

pub fn execute_search(engine: &SearchEngine, query: &String) -> Result<String, String> {
    return match engine {
        SearchEngine::Google => {
            println!(
                "\n{}: Searching using {}...",
                query.yellow(),
                engine.to_string()
            );
            open::that(PathBuf::from(format!(
                "https://www.google.com/search?q={}",
                query.as_str()
            )))
            .expect("Couldn't launch Google!");
            Ok(format!("Opened '{}' with Google", query.as_str()))
        }
        SearchEngine::DuckDuckGo => {
            println!(
                "\n{}: Searching using {}...",
                query.yellow(),
                engine.to_string()
            );
            open::that(PathBuf::from(format!(
                "https://duckduckgo.com/?t=ffab&q={}",
                query.as_str()
            )))
            .expect("Couldn't launch DuckDuckGo!");
            Ok(format!("Opened '{}' with DuckDuckGo", query.as_str()))
        }
        SearchEngine::Perplexity => {
            println!(
                "\n{}: Searching using {}...",
                query.yellow(),
                engine.to_string()
            );
            open::that(PathBuf::from(format!(
                "https://www.perplexity.ai/search?q={}",
                query.as_str()
            )))
            .expect("Couldn't launch Perplexity!");
            Ok(format!("Opened '{}' with Perplexity", query.as_str()))
        }
        SearchEngine::ChatGPT => {
            println!(
                "\n{}: Searching using {}...",
                query.yellow(),
                engine.to_string()
            );
            open::that(PathBuf::from(format!(
                "https://chatgpt.com/?q={}",
                query.as_str()
            )))
            .expect("Couldn't launch ChatGPT!");
            Ok(format!("Opened '{}' with Perplexity", query.as_str()))
        } // _ => {
          //     println!(
          //         "\n{} > Unknown search engine: {}...",
          //         "ERROR".red(),
          //         engine.to_string()
          //     );
          //     Ok(format!(
          //         "Error: Unknown search engine: {}",
          //         engine.to_string().red()
          //     ))
          // }
    };
}

// ------------------------------------------------------
// ---------------------FAVORITES------------------------
// ------------------------------------------------------

pub fn execute_fav_view(favorites_manager: &mut FavoritesManager) -> Result<String, String> {
    let states = favorites_manager.get_all();
    if states.is_empty() {
        println!(
            "No favorites found! Add a favorite by using {}",
            "FAV SET STATE".yellow()
        );
        return Ok(String::from(
            "No matches found! Try switching root directories.",
        ));
    }
    println!("\nFavorites List:");
    for (index, state) in states.iter().enumerate() {
        println!(
            "{}: {} > {} ",
            index.to_string().bright_blue(),
            state.get_alias_name().yellow(),
            state.get_path().to_string_lossy()
        )
    }
    println!(
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
            println!(
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
        println!(
            "Added STATE {} to Favorites. Use FAV VIEW to view Favorites list.",
            current_state.display().to_string().green()
        );
        return Ok(String::from("Completed FAV SET"));
    }
    if favs.len() + 1 > 10 {
        println!("ERROR: Favorites List is full! (Max Favorites = 10)");
        return Ok(String::from("FAV SET TOO_MANY!"));
    } else {
        let new_fav = Favorite::from(current_state.clone());
        favorites_manager
            .add(new_fav)
            .expect("Couldn't add favorites to Favorite Manager!");
        println!(
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
        println!("ERROR: Index out of bounds!");
        return Ok(String::from("Invalid index!"));
    }
    match favorites_manager.remove(index) {
        Ok(_) => println!("Removed favorite from FavoritesManager!"),
        Err(msg) => {
            return Err(format!(
                "Failed to remove favorite from FavoritesManager: {}",
                msg
            ));
        }
    };

    return Ok("Removed favorites from Favorite Manager!".to_string());
}
