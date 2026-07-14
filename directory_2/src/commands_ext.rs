use crate::commands::execute_command_legacy;
use crate::favorites::FavoritesManager;
use crate::file_system_state::FileSystemState;
use crate::parser::Command;
use crate::value::Value;
use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub async fn execute_command(
    command: Command,
    input: Option<Value>,
    file_system_state: &mut FileSystemState,
    favorites_manager: &mut FavoritesManager,
) -> Result<Value, String> {
    match command {
        Command::Filter { column, operator, value } => {
            execute_filter(input, &column, &operator, &value)
        }
        Command::SelectFields { fields } => {
            execute_select_fields(input, fields)
        }
        Command::ListCommands => {
            execute_list_commands_structured()
        }
        Command::ListDirectory { show_hidden, detailed } => {
            execute_list_directory_structured(file_system_state, show_hidden, detailed)
        }
        Command::Env => {
            execute_env_structured()
        }
        // Fallback to legacy strings wrapped in Value
        other => {
            let res = execute_command_legacy(other, file_system_state, favorites_manager).await?;
            Ok(Value::String(res))
        }
    }
}

fn execute_list_directory_structured(
    sys_state: &mut FileSystemState,
    show_hidden: bool,
    detailed: bool,
) -> Result<Value, String> {
    let current_path = sys_state.get_current_path();
    if !current_path.is_dir() {
        return Err(String::from("Not a Directory"));
    }

    let mut entries: Vec<_> = current_path
        .read_dir()
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .collect();

    entries.sort_by(|a, b| a.path().file_name().cmp(&b.path().file_name()));

    let mut rows = Vec::new();

    for entry in entries {
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

        if !show_hidden && name.starts_with('.') {
            continue;
        }

        let metadata = entry.metadata().ok();
        let is_dir = path.is_dir();

        let mut row = HashMap::new();
        row.insert("Name".to_string(), Value::String(name));
        row.insert("Type".to_string(), Value::String(if is_dir { "Dir".to_string() } else { "File".to_string() }));

        if let Some(m) = &metadata {
            if is_dir {
                row.insert("Size".to_string(), Value::String("-".to_string()));
            } else {
                row.insert("Size".to_string(), Value::Integer(m.len() as i64));
            }

            if let Ok(sys_time) = m.modified() {
                let datetime: chrono::DateTime<chrono::Local> = sys_time.into();
                row.insert("Modified".to_string(), Value::String(datetime.format("%Y-%m-%d %H:%M").to_string()));
            } else {
                row.insert("Modified".to_string(), Value::String("-".to_string()));
            }

            #[cfg(unix)]
            {
                let perms = m.permissions().mode();
                row.insert("Perms".to_string(), Value::String(format!("{:o}", perms & 0o777)));
            }
        }

        rows.push(row);
    }

    Ok(Value::Table(rows))
}

fn execute_env_structured() -> Result<Value, String> {
    let mut map = HashMap::new();
    for (key, value) in std::env::vars() {
        map.insert(key, Value::String(value));
    }
    Ok(Value::Record(map))
}

fn execute_list_commands_structured() -> Result<Value, String> {
    let mut rows = Vec::new();

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

    let mut add_commands = |category: &str, cmds: &[(&str, &str)]| {
        for (cmd, desc) in cmds {
            let mut row = HashMap::new();
            row.insert("Category".to_string(), Value::String(category.to_string()));
            row.insert("Command".to_string(), Value::String(cmd.to_string()));
            row.insert("Description".to_string(), Value::String(desc.to_string()));
            rows.push(row);
        }
    };

    add_commands("Meta", &meta_commands);
    add_commands("Environment", &env_commands);
    add_commands("Alias", &alias_commands);
    add_commands("TUI Configuration", &tui_commands);
    add_commands("Directory/File", &dir_file_commands);
    add_commands("State", &state_commands);
    add_commands("Favorites", &fav_commands);
    add_commands("Search", &search_commands);

    Ok(Value::Table(rows))
}

fn execute_filter(input: Option<Value>, column: &str, operator: &str, value: &str) -> Result<Value, String> {
    let input = input.ok_or_else(|| "FILTER requires an input pipeline".to_string())?;
    match input {
        Value::Table(rows) => {
            let mut filtered = Vec::new();
            for row in rows {
                // Find matching column case-insensitively
                let mut cell_val_opt = None;
                for (k, v) in &row {
                    if k.eq_ignore_ascii_case(column) {
                        cell_val_opt = Some(v);
                        break;
                    }
                }
                
                if let Some(cell_val) = cell_val_opt {
                    let cell_str = cell_val.to_string();
                    let matches = match operator {
                        "=" | "==" => cell_str == value,
                        "!=" => cell_str != value,
                        ">" => {
                            if let (Ok(c), Ok(v)) = (cell_str.parse::<f64>(), value.parse::<f64>()) { c > v } else { false }
                        }
                        "<" => {
                            if let (Ok(c), Ok(v)) = (cell_str.parse::<f64>(), value.parse::<f64>()) { c < v } else { false }
                        }
                        ">=" => {
                            if let (Ok(c), Ok(v)) = (cell_str.parse::<f64>(), value.parse::<f64>()) { c >= v } else { false }
                        }
                        "<=" => {
                            if let (Ok(c), Ok(v)) = (cell_str.parse::<f64>(), value.parse::<f64>()) { c <= v } else { false }
                        }
                        "CONTAINS" => cell_str.contains(value),
                        _ => return Err(format!("Unsupported operator: {}", operator)),
                    };
                    if matches {
                        filtered.push(row);
                    }
                }
            }
            Ok(Value::Table(filtered))
        }
        _ => Err("FILTER only works on Tables".to_string()),
    }
}

fn execute_select_fields(input: Option<Value>, fields: Vec<String>) -> Result<Value, String> {
    let input = input.ok_or_else(|| "SELECT requires an input pipeline".to_string())?;
    match input {
        Value::Table(rows) => {
            let mut selected = Vec::new();
            for row in rows {
                let mut new_row = HashMap::new();
                for field in &fields {
                    // Find matching column case-insensitively
                    for (k, v) in &row {
                        if k.eq_ignore_ascii_case(field) {
                            new_row.insert(k.clone(), v.clone());
                            break;
                        }
                    }
                }
                selected.push(new_row);
            }
            Ok(Value::Table(selected))
        }
        _ => Err("SELECT only works on Tables".to_string()),
    }
}
