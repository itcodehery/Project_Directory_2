use crate::{
    commands_ext::execute_command,
    favorites::FavoritesManager,
    file_system_state::FileSystemState,
    parser::parse_command,
};
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::process::Stdio;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

pub async fn run_shell(
    mut sys_state: FileSystemState,
    mut fav_manager: FavoritesManager,
) -> Result<(), Box<dyn std::error::Error>> {
    // Print the welcome message
    println!("{}", "---------------------------".green());
    println!("{}", "DIR2 Shell (True Shell Mode)".bright_green().bold());
    println!("{}", "Welcome to the true shell experience!".green());
    println!("{}", "---------------------------".green());

    let mut rl = rustyline::Editor::<crate::completion::Dir2Helper, rustyline::history::DefaultHistory>::new()?;
    rl.set_helper(Some(crate::completion::Dir2Helper::new()));
    // let _ = rl.load_history("history.txt");

    loop {
        // Build the prompt
        let current_path = sys_state.get_current_path().to_string_lossy().to_string();
        let prompt = format!("{} {} ", "[dir2]".bright_green().bold(), current_path.blue());

        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                rl.add_history_entry(line)?;

                if line.to_uppercase() == "EXIT" || line.to_uppercase() == "QUIT" {
                    break;
                }

                let mut expanded_cmd = crate::utils::substitute_env_vars(line);
                expanded_cmd = sys_state.expand_aliases(&expanded_cmd);

                match parse_command(&expanded_cmd) {
                    Ok(crate::parser::Command::Unknown { command, args }) => {
                        let is_background = args.last().map(|s| s == "&").unwrap_or(false);
                        let args_filtered = if is_background {
                            &args[..args.len() - 1]
                        } else {
                            &args[..]
                        };

                        if is_background {
                            let child_res = tokio::process::Command::new(&command)
                                .args(args_filtered)
                                .current_dir(sys_state.get_current_path())
                                .stdout(Stdio::null())
                                .stderr(Stdio::null())
                                .spawn();
                                
                            match child_res {
                                Ok(child) => {
                                    let child_arc = Arc::new(TokioMutex::new(child));
                                    let job_id = crate::jobs::add_job(command.clone(), child_arc.clone());
                                    println!("[{}] Background Job Started: {}", job_id, command);
                                    
                                    tokio::spawn(async move {
                                        loop {
                                            let mut exited = false;
                                            {
                                                let mut c_guard = child_arc.lock().await;
                                                if let Ok(Some(_)) = c_guard.try_wait() {
                                                    exited = true;
                                                }
                                            }
                                            if exited {
                                                break;
                                            }
                                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                        }
                                        crate::jobs::remove_job(job_id);
                                        println!("\n[{}] Job Finished", job_id);
                                    });
                                }
                                Err(e) => {
                                    println!("Failed to start background task: {}", e);
                                }
                            }
                        } else {
                            let mut child = std::process::Command::new(&command)
                                .args(args_filtered)
                                .current_dir(sys_state.get_current_path())
                                .spawn();

                            if let Ok(mut c) = child {
                                let _ = c.wait();
                            } else {
                                println!("Command not found or failed to execute: {}", command);
                            }
                        }
                    }
                    Ok(crate::parser::Command::Pipe { commands, output_file }) => {
                        crate::pipe_executor::execute_pipeline(commands, output_file, &mut sys_state, &mut fav_manager).await;
                    }
                    Ok(crate::parser::Command::Jobs) => {
                        let jobs = crate::jobs::list_jobs();
                        if jobs.is_empty() {
                            println!("No active background jobs.");
                        } else {
                            println!("Background Jobs:");
                            for (id, cmd) in jobs {
                                println!("[{}] {}", id, cmd);
                            }
                        }
                    }
                    Ok(crate::parser::Command::Fg { id }) => {
                        if let Some(job) = crate::jobs::get_job(id) {
                            println!("Bringing job [{}] to foreground...", id);
                            let mut child = job.child.lock().await;
                            let _ = child.wait().await;
                            crate::jobs::remove_job(id);
                            println!("[{}] Job Finished", id);
                        } else {
                            println!("Job ID {} not found.", id);
                        }
                    }
                    Ok(crate::parser::Command::Kill { id }) => {
                        if let Some(job) = crate::jobs::get_job(id) {
                            let mut child = job.child.lock().await;
                            let _ = child.kill().await;
                            crate::jobs::remove_job(id);
                            println!("[{}] Job Killed", id);
                        } else {
                            println!("Job ID {} not found.", id);
                        }
                    }
                    Ok(crate::parser::Command::ClearScreen) => {
                        let _ = rl.clear_screen();
                    }
                    Ok(command) => {
                        // Execute known dir2 commands
                        match execute_command(command, None, &mut sys_state, &mut fav_manager).await {
                            Ok(output) => {
                                let out_str = output.to_string();
                                if !out_str.is_empty() {
                                    println!("{}", out_str);
                                }
                            }
                            Err(e) => {
                                println!("Error: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Parse Error: {}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    
    // let _ = rl.save_history("history.txt");
    Ok(())
}
