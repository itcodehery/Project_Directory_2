use crate::parser::{parse_command, Command};
use crate::commands_ext::execute_command;
use crate::favorites::FavoritesManager;
use crate::file_system_state::FileSystemState;
use crate::value::Value;
use std::process::{Command as OsCommand, Stdio};
use std::io::Write;
use std::fs::File;

pub async fn execute_pipeline(
    commands: Vec<Vec<String>>,
    output_file: Option<String>,
    sys_state: &mut FileSystemState,
    fav_manager: &mut FavoritesManager,
) {
    let mut current_value: Option<Value> = None;

    for (i, cmd_tokens) in commands.iter().enumerate() {
        if cmd_tokens.is_empty() {
            continue;
        }

        let cmd_str = cmd_tokens.join(" ");
        let parsed_cmd = match parse_command(&cmd_str) {
            Ok(cmd) => cmd,
            Err(e) => {
                println!("Failed to parse command in pipe: {}", e);
                return;
            }
        };

        match parsed_cmd {
            Command::Unknown { command, args } => {
                let mut os_cmd = OsCommand::new(&command);
                os_cmd.args(&args);
                os_cmd.current_dir(sys_state.get_current_path());
                
                os_cmd.stdin(Stdio::piped());
                os_cmd.stdout(Stdio::piped());
                
                match os_cmd.spawn() {
                    Ok(mut child) => {
                        if let Some(val) = current_value.take() {
                            if let Some(mut stdin) = child.stdin.take() {
                                let _ = stdin.write_all(val.to_string().as_bytes());
                            }
                        }
                        
                        match child.wait_with_output() {
                            Ok(output) => {
                                let out_str = String::from_utf8_lossy(&output.stdout).to_string();
                                current_value = Some(Value::String(out_str));
                            }
                            Err(e) => {
                                println!("Error waiting for external command: {}", e);
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to spawn external command '{}': {}", command, e);
                        return;
                    }
                }
            }
            internal_cmd => {
                match execute_command(internal_cmd, current_value.take(), sys_state, fav_manager).await {
                    Ok(val) => {
                        current_value = Some(val);
                    }
                    Err(e) => {
                        println!("Pipeline error: {}", e);
                        return;
                    }
                }
            }
        }
    }

    if let Some(val) = current_value {
        let out_str = val.to_string();
        if let Some(file_path) = output_file {
            let path = sys_state.get_current_path().join(file_path);
            if let Ok(mut f) = File::create(path) {
                let _ = f.write_all(out_str.as_bytes());
            } else {
                println!("Failed to write to output file");
            }
        } else {
            if !out_str.is_empty() {
                println!("{}", out_str);
            }
        }
    }
}
