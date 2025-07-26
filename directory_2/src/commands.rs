use std::path::PathBuf;
use crate::file_system_state::FileSystemState;
use crate::filesystem::{is_dir, is_dir_empty, is_dir_the_root, resolve_and_normalize};
use crate::parser::Command;

pub fn execute_command(command:Command, file_system_state: &mut FileSystemState) -> Result<String, String> {
    match command {
        Command::ListCommands => execute_list_all_cmd(),
        Command::DodgeDirectory => {
            execute_dodge_directory(file_system_state)
        },
        Command::WatchDirectory { directory: _directory } => {
            return Ok(String::from("Executed: Watch Directory"))
        },
        Command::ClearScreen => {
            println!("\x1B[2J\x1B[1;1H");
            return Ok(String::from("Executed: Clear Screen"))
        },
        Command::Exit => {Ok("exited!".to_string())},
        Command::Select { filename: _filename, directory: _directory } => {
            return Ok(String::from("Executed: Select"))
        },
        Command::ViewState => {
            execute_view_state(file_system_state)
            // return Ok(String::from("Executed: View State"))
        },
        Command::MetaState => {
            return Ok(String::from("Executed: Meta State"))
        },
        Command::FindExact { filename: _filename } => {
            return Ok(String::from("Executed: Find Exact"))
        },
        Command::RunState => {
            return Ok(String::from("Executed: Run State"))
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
            return Ok(String::from("Unexecuted: Unknown command"))
        },
        _ => {
            return Err(String::from("Error: Unknown command"))
        }
    }
}
pub fn execute_list_all_cmd() ->Result<String, String>{
    let command_list = [
        "DIR2 Commands (All Case-insensitive)",
        "---------------------",
        "\nMeta Commands:",
        "LS : Lists Commands",
        "DD : Dodge Directory",
        "WD : Watch Directory",
        "CLS | /C : Clear Screen",
        "EXIT | /E : Exit Terminal",
        "\nSTATE Commands:",
        "SELECT <filename.ext> FROM <directory> : Sets <filename.ext> file as current STATE.",
        "VIEW STATE | VS: To view current STATE.",
        "META STATE | MS: To view current STATE File Metadata.",
        "FIND EXACT <query> | FE <query> : Finds file by performing a system-wide search and stores it in the STATE.",
        "RUN STATE | RS : Runs the file or script present in the current STATE.",
        "\nFavorites Commands:",
        "FAV VIEW : View all Favorites as a List.",
        "FAV SET STATE : Sets current state as latest favorite.",
        "FAV RM <filename> : Removes <filename> from favorites.",
        "RUN FAV <index> : Runs the file at the index of the Favorites list.",
        "---------------------",
    ];

    for command in command_list {
        println!("{}", command);
    }

    Ok("Executed: List Commands".to_string())
}

pub fn execute_dodge_directory(sys_state: &mut FileSystemState) -> Result<String, String> {
    let current_path = sys_state.get_current_path();
    let base_directory = std::env::current_dir().unwrap();
    // Check if the path is a directory or a file
    if !is_dir(current_path) {
        return Err(String::from("Not a Directory"))
    }
    // Check if the directory is the root
    if is_dir_the_root(current_path) {
        return Err(String::from("Cannot Dodge Root Directory"))
    }
    sys_state.set_current_directory(current_path.parent().unwrap().to_path_buf());
    Ok(String::from("Executed: Dodge Directory"))
    // return Err(String::from("Can't execute Dodge Directory"));
}

pub fn execute_view_state(sys_state: &mut FileSystemState) -> Result<String, String> {
    let current_path = sys_state.get_current_path();
    let current_state = sys_state.get_current_state();

    println!("Current STATE:\nState: {:?}", current_state);
    println!("Base directory: {:?}", current_path);

    return Ok(String::from("Executed View State!"));
}

// pub fn execute_watch_directory(sys_state: &mut FileSystemState) -> Result<String, String> {
//     let current_path = sys_state.get_current_path();
//
//
// }