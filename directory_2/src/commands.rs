use std::path::{Path, PathBuf};
use colored::Colorize;
use crate::file_system_state::FileSystemState;
use crate::filesystem;
use crate::filesystem::{get_directory_without_parent, get_file_metadata, is_dir, is_dir_the_root, is_executable};
use crate::parser::Command;

pub fn execute_command(command:Command, file_system_state: &mut FileSystemState) -> Result<String, String> {
    match command {
        Command::ListCommands => execute_list_all_cmd(),
        Command::DodgeDirectory => {
            execute_dodge_directory(file_system_state)
        },
        Command::WatchDirectory { directory: _directory } => {
            execute_watch_directory(file_system_state, &_directory)
        },
        Command::ListDirectory => {
    execute_list_directory(file_system_state)
        }
        Command::ClearScreen => {
            println!("\x1B[2J\x1B[1;1H");
            return Ok(String::from("Executed: Clear Screen"))
        },
        Command::Exit => {Ok("exited!".to_string())},
        Command::Select { filename: _filename, directory: _directory } => {
            // return Ok(String::from("Executed: Select"))
            execute_select(file_system_state, _filename,_directory)
        },
        Command::ViewState => {
            execute_view_state(file_system_state)
            // return Ok(String::from("Executed: View State"))
        },
        Command::ClearState => {
            execute_clear_state(file_system_state)
        },
        Command::MetaState => {
            execute_meta_state(file_system_state)
            // return Ok(String::from("Executed: Meta State"))
        },
        Command::FindExact { filename: _filename } => {
            return Ok(String::from("Executed: Find Exact"))
        },
        Command::RunState => {
            execute_run_state(file_system_state)
            // return Ok(String::from("Executed: Run State"))
        },
        Command::FavView => {
            return Ok(String::from("Executed: Fav View"))
        },
        Command::FavRm { filename: _filename } => {
            return Ok(String::from("Executed: Fav Rm"))
        },
        Command::FavSet => {
            return Ok(String::from("Executed: Fav Set"))
        },
        Command::RunFav { index: _index } => {
            return Ok(String::from("Executed: Run Fav"))
        },
        Command::Unknown {command} => {
            return Ok(String::from("Unexecuted: Unknown command {command}"))
        },
        _ => {
            return Err(String::from("Error: Unknown command"))
        }
    }
}
pub fn execute_list_all_cmd() ->Result<String, String> {
    let titles_list = [
        "DIR2 Commands (All Case-insensitive)",
        "---------------------",
    ];

    use std::collections::HashMap;

    let titles_list = [
        "DIR2 Commands (All Case-insensitive)",
        "---------------------",
        "Meta Commands:",
        "State Commands:",
        "Favorites Commands:",
        "---------------------",
    ];

    let meta_commands: HashMap<&str, &str> = [
        ("LS", "Lists Commands"),
        ("DD", "Dodge Directory"),
        ("WD", "Watch Directory"),
        ("CLS | /C", "Clear Screen"),
        ("EXIT | /E", "Exit Terminal"),
    ].iter().cloned().collect();

    let state_commands: HashMap<&str, &str> = [
        ("SELECT <filename.ext> FROM <directory>", "Sets <filename.ext> file as current STATE"),
        ("VIEW STATE | VS", "To view current STATE"),
        ("META STATE | MS", "To view current STATE File Metadata"),
        ("FIND EXACT <query> | FE <query>", "Finds file by performing a system-wide search and stores it in the STATE"),
        ("RUN STATE | RS", "Runs the file or script present in the current STATE"),
    ].iter().cloned().collect();

    let fav_commands: HashMap<&str, &str> = [
        ("FAV VIEW", "View all Favorites as a List"),
        ("FAV SET STATE", "Sets current state as latest favorite"),
        ("FAV RM <filename>", "Removes <filename> from favorites"),
        ("RUN FAV <index>", "Runs the file at the index of the Favorites list"),
    ].iter().cloned().collect();

    println!("\n{}\n{}", titles_list[0], titles_list[1]);
    println!("{}",titles_list[2]);
    for command in meta_commands.keys() {
        println!("{} : {}",command.bright_blue(), meta_commands[command]);
    }
    println!("\n{}",titles_list[3]);
    for command in state_commands.keys() {
        println!("{} : {}", command.yellow(), state_commands[command]);
    }
    println!("\n{}",titles_list[4]);
    for command in fav_commands.keys() {
        println!("{} : {}", command.bright_green(), fav_commands[command]);
    }
    println!("\n");

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
            println!("Error: {}","Cannot dodge Root Directory!".red().to_string());
            Ok(String::from("Failed: Dodge Directory"))
        }
    }
    // Err(String::from("Cannot navigate to parent directory"))
}

pub fn execute_list_directory(sys_state: &mut FileSystemState) -> Result<String, String> {
    let current_path = sys_state.get_current_path();

    if !is_dir(current_path) {
        return Err(String::from("Not a Directory"))
    }
    else {
        for entry in current_path.read_dir().expect("read_dir call failed") {
            if let Ok(entry) = entry {
                println!("{}", get_directory_without_parent(&*entry.path()));
            }
        }
    }
    return Ok(String::from("Executed: List Directory"));
}

pub fn execute_select(sys_state: &mut FileSystemState, filename: String, directory: String) -> Result<String, String> {
    let dir_path : PathBuf = if directory.is_empty() || directory == "/" {
        sys_state.get_current_path().clone()
    } else {
        let path = Path::new(&directory);
        if path.is_absolute() {
            path.to_path_buf()
        }
        else {
            filesystem::resolve_path(path, sys_state.get_current_path())
        }
    };

    if !filesystem::path_exists(&dir_path) {
        return Err(format!("Directory {} does not exist!", directory));
    }
    if !filesystem::is_dir(&dir_path) {
        return Err(format!("Directory {} is not a directory!", directory));
    }

    sys_state.set_current_directory(dir_path.clone());

    let file_path = dir_path.join(&filename);

    if !filesystem::path_exists(&file_path) {
        return Err(format!("File {} does not exist in Directory {}!", file_path.display(), directory));
    }
    if !filesystem::is_file(&file_path) {
        return Err(format!("File {} is not a file!", file_path.display()));
    }
    sys_state.set_current_state(file_path.clone());

    println!("Selected STATE: {}",file_path.display().to_string().green());
    return Ok(format!("Selected {}", file_path.display()));
}

pub fn execute_run_state(sys_state: &mut FileSystemState) -> Result<String, String> {
    // Get the selected file
    let file_path = match sys_state.get_current_state() {
        Some(path) => path,
        None => {
            println!("\nError: {}","No file selected. Use SELECT command first".red().to_string());
            return Ok(String::from("No file selected. Use SELECT command first."))
        },
    };

    // Validate file still exists and is executable
    if !filesystem::path_exists(file_path) {
        println!("\nError: {}","Selected file no longer exists".red().to_string());
        return Ok(format!("Error: Selected file '{}' no longer exists", file_path.display().to_string().red().to_string()));
    }

    if !filesystem::is_executable(file_path) {
        // For non-executable files, open with default application
        match std::process::Command::new("cmd")
            .args(["/c", "start", "", &file_path.to_string_lossy()])
            .spawn()
        {
            Ok(_child) => {
                println!("Opening STATE with default application: {}", file_path.display());
                Ok(format!("Opened '{}' with default application", file_path.display()))
            }
            Err(e) => {
                println!("\nError: {} -> {}","Failed to open STATE".red().to_string(), file_path.display().to_string().red());
                Ok(format!("Failed to open STATE: '{}': {}", file_path.display().to_string().red(), e)) }
        }
    } else {
        match std::process::Command::new(file_path).spawn() {
            Ok(_child) => {
                println!("Running STATE: {}", file_path.display());
                Ok(format!("Started: {}", file_path.display()))
            }
            Err(e) => {
                println!("\nError: {} -> {}","Failed to run STATE".red().to_string(),e.to_string().red().to_string());
                Ok(format!("Failed to run STATE: {}", e.to_string().red().to_string())) },
        }
    }
}

pub fn execute_view_state(sys_state: &mut FileSystemState) -> Result<String, String> {
    let current_state = sys_state.get_current_state();
    if current_state.is_none() {
        println!("{}:\nState: None","Current STATE".yellow());
    }
    else {
        println!("{}:\nState: {}","Current STATE".yellow(), current_state.clone().unwrap().to_str().unwrap());
    }
    return Ok(String::from("Executed View State!"));
}

pub fn execute_clear_state(sys_state: &mut FileSystemState) -> Result<String, String> {
    sys_state.clear_state();
    println!("{}", "STATE Dropped".yellow());
    return Ok(String::from("Executed Clear State!"));
}

pub fn execute_watch_directory(sys_state: &mut FileSystemState, directory: &String) -> Result<String, String> {
    // println!("DEBUG: Looking for directory: '{}'", directory);
    // println!("DEBUG: Current path: '{}'", sys_state.get_current_path().display());
    let dir_path = PathBuf::from(directory);
    // println!("DEBUG: Full path to check: '{}'", dir_path.display());
    // println!("DEBUG: Current Directory according to the system: {} ", std::env::current_dir().unwrap().to_str().unwrap());
    // println!("DEBUG: Path exists: {}", dir_path.exists());
    // println!("DEBUG: Is directory: {}", dir_path.is_dir());
    // Check if directory exists
    if !filesystem::path_exists(&dir_path) {
        println!("Error: Directory {} does not exist!", directory.red().to_string());
        return Ok(format!("Directory '{}' does not exist", directory));
    }

    // Check if it's actually a directory
    if !filesystem::is_dir(&dir_path) {
        println!("Error: {} is not a directory!", directory.red().to_string());
        return Ok(format!("'{}' is not a directory", directory));
    }

    // Update current directory
    sys_state.set_current_directory(sys_state.get_current_path().join(dir_path));
    Ok(format!("Changed to directory: {}", directory))
}
pub fn execute_meta_state(sys_state: &mut FileSystemState) -> Result<String, String> {
    let current_state = sys_state.get_current_state();

    if current_state.is_none() {
        println!("Error: STATE is Empty!");
        return Ok(String::from("STATE is Empty!"));
    }
    else {
        println!("\n{}\n {:?}", "STATE Metadata:".yellow(), get_file_metadata(current_state.as_ref().unwrap()));
        return Ok(String::from("Executed: STATE Metadata"));
    }
}