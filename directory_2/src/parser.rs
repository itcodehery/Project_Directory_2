// use std::fmt::format;

use colored::Colorize;

use crate::search::SearchEngine;

#[derive(Debug)]
pub enum Command {
    // Meta Commands
    ListCommands,
    ClearScreen,
    Config,
    History,
    Exit,
    Docs {
        command_name: Option<String>,
    },
    SqlQuery {
        query: String,
    },
    Unknown {
        command: String,
        args: Vec<String>,
    },
    
    // Environment Commands
    Export {
        key: String,
        value: String,
    },
    Unset {
        key: String,
    },
    Env,
    Echo {
        text: String,
    },
    
    // Alias Commands
    Alias {
        key: String,
        value: String,
    },
    Unalias {
        key: String,
    },
    Aliases,

    // Interactive Config Commands
    AddInteractive {
        command: String,
    },
    RemoveInteractive {
        command: String,
    },
    ListInteractive,

    // State Management Commands
    Select {
        filename: String,
        directory: String,
    },
    ViewState,
    ClearState,
    RunState,
    MetaState,

    // Directory Management Commands
    DodgeDirectory,
    WatchDirectory {
        directory: String,
    },
    ListDirectory {
        show_hidden: bool,
    },
    ChangeDrive {
        drive: String,
    },
    MakeDirectory {
        directory: String,
    },
    RemoveDirectory {
        directory: String,
    },
    RenameDirectory {
        old_directory: String,
        new_directory: String,
    },
    // File Management Commands
    MakeFile {
        filename: String,
    },
    RemoveFile {
        filename: String,
    },
    RenameFile {
        old_filename: String,
        new_filename: String,
    },

    // Search Commands
    Search {
        engine: String,
        query: String,
    },

    // Favorite Commands
    FavView,
    FavSet,
    FavRm {
        index: usize,
    },
    RunFav {
        index: usize,
    },
}

pub fn parse_command(input: &str) -> Result<Command, String> {
    if input.trim().is_empty() {
        return Err(String::from("Empty command"));
    }
    
    // Try SQL Parser First
    let dialect = sqlparser::dialect::GenericDialect {};
    if sqlparser::parser::Parser::parse_sql(&dialect, input).is_ok() {
        return Ok(Command::SqlQuery {
            query: input.to_string(),
        });
    }

    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err(String::from("Empty command"));
    }

    return match tokens[0].to_uppercase().as_str() {
        // Meta Commands
        "LC" | "LIST COMMANDS" => Ok(Command::ListCommands),
        "CLS" | "/C" | "CLEAR" => Ok(Command::ClearScreen),
        "CONFIG" | "RC" => Ok(Command::Config),
        "HISTORY" | "HIST" => Ok(Command::History),
        "DOCS" | "MAN" => {
            let cmd = if tokens.len() > 1 {
                Some(tokens[1].clone())
            } else {
                None
            };
            Ok(Command::Docs { command_name: cmd })
        }
        "EXIT" | "QUIT" | "/Q" => Ok(Command::Exit),
        "EXPORT" => {
            if tokens.len() < 2 {
                return Err(String::from("Missing argument. Usage: export VAR=value"));
            }
            let arg = tokens[1..].join(" ");
            let parts: Vec<&str> = arg.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(String::from("Invalid format. Usage: export VAR=value"));
            }
            Ok(Command::Export {
                key: parts[0].to_string(),
                value: parts[1].to_string(),
            })
        }
        "UNSET" => {
            if tokens.len() != 2 {
                return Err(String::from("Missing argument. Usage: unset VAR"));
            }
            Ok(Command::Unset {
                key: tokens[1].to_string(),
            })
        }
        "ENV" => Ok(Command::Env),
        "ECHO" => {
            let text = if tokens.len() > 1 {
                tokens[1..].join(" ")
            } else {
                String::new()
            };
            Ok(Command::Echo { text })
        }
        "ALIAS" => {
            if tokens.len() < 2 {
                return Ok(Command::Aliases);
            }
            let arg = tokens[1..].join(" ");
            let parts: Vec<&str> = arg.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(String::from("Invalid format. Usage: alias name='command'"));
            }
            Ok(Command::Alias {
                key: parts[0].to_string(),
                value: parts[1].to_string(),
            })
        }
        "UNALIAS" => {
            if tokens.len() != 2 {
                return Err(String::from("Missing argument. Usage: unalias name"));
            }
            Ok(Command::Unalias {
                key: tokens[1].to_string(),
            })
        }
        "ALIASES" => Ok(Command::Aliases),
        "S" | "SEARCH" => {
            if tokens.len() < 3 {
                return Err(String::from("Missing arguments. Usage: S <engine> <query>"));
            }
            Ok(Command::Search {
                engine: tokens[1].to_string(),
                query: tokens[2..].join(" "),
            })
        }
        "TUIADD" => {
            if tokens.len() != 2 {
                return Err(String::from("Missing argument. Usage: TUIADD <command>"));
            }
            Ok(Command::AddInteractive {
                command: tokens[1].to_string(),
            })
        }
        "TUIRM" => {
            if tokens.len() != 2 {
                return Err(String::from("Missing argument. Usage: TUIRM <command>"));
            }
            Ok(Command::RemoveInteractive {
                command: tokens[1].to_string(),
            })
        }
        "TUILS" => Ok(Command::ListInteractive),
        // Directory Commands
        "DD" => Ok(Command::DodgeDirectory),
        "WD" => parse_watch_directory(&tokens),
        "LD" | "LS" | "LL" | "LA" => parse_list_directory(&tokens),
        "CD" => parse_change_drive(&tokens),
        "MKDIR" => parse_make_directory(&tokens),
        "RMDIR" => parse_remove_directory(&tokens),
        "RENDIR" => parse_rename_directory(&tokens),
        // File Management Commands
        "MKFILE" => parse_make_file(&tokens),
        "RMFILE" => parse_remove_file(&tokens),
        "RENFILE" => parse_rename_file(&tokens),

        // STATE Commands
        "SELECT" => parse_select(&tokens),
        "VIEW" | "VS" => parse_view(&tokens),
        "DROP" | "DS" => parse_drop_state(&tokens),
        "RUN" | "RS" => parse_run(&tokens),
        "META" => parse_meta_state(&tokens),
        // Favorite Commands
        "RF" => parse_run(&tokens),
        "FAV" => parse_fav(&tokens),
        // Search Commands
        _ => parse_unknown(&tokens),
    };
}

fn tokenize(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_quotes = false;

    // Create peekable iterator over input characters
    let mut chars = input.chars().peekable();
    // For example, in a tokenizer, you might want to keep consuming characters until you
    // hit a whitespace or special character.
    // The peek() method lets you check if the next character is a whitespace
    // or special character before advancing the iterator.

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
            }
            ' ' | '\t' => {
                if in_quotes {
                    current_token.push(ch);
                } else if !current_token.is_empty() {
                    tokens.push(current_token.trim().to_string());
                    current_token.clear();
                }
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    if !current_token.is_empty() {
        tokens.push(current_token.trim().to_string());
    }

    if in_quotes {
        return Err("Unclosed quotes in command.".red().to_string());
    }

    return Ok(tokens);
}

fn parse_watch_directory(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() < 2 {
        return Err("Expected <directory> AFTER WD".red().to_string());
    }

    let directory = if tokens.len() == 2 {
        // Single token, parse normally
        parse_filename(tokens[1].clone())
    } else {
        // Multiple tokens, join them with spaces
        tokens[1..].join(" ")
    };

    Ok(Command::WatchDirectory { directory })
}

fn parse_list_directory(tokens: &[String]) -> Result<Command, String> {
    let mut show_hidden = false;
    let base_cmd = tokens[0].to_uppercase();
    if base_cmd == "LA" || base_cmd == "LL" {
        show_hidden = true;
    }
    for token in tokens.iter().skip(1) {
        if token.starts_with('-') && token.to_lowercase().contains('a') {
            show_hidden = true;
        }
    }
    return Ok(Command::ListDirectory { show_hidden });
}

fn parse_make_directory(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() != 2 {
        return Err(format!("Expected {}", "MKDIR <directory>".red()));
    }
    let directory = parse_filename(tokens[1].clone());
    return Ok(Command::MakeDirectory { directory });
}

fn parse_remove_directory(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() != 2 {
        return Err(format!("Expected {}", "RMDIR <directory>".red()));
    }
    let directory = parse_filename(tokens[1].clone());
    return Ok(Command::RemoveDirectory { directory });
}

fn parse_rename_directory(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() != 3 {
        return Err(format!(
            "Expected {}",
            "RENDIR <old_directory> <new_directory>".red()
        ));
    }
    let old_directory = parse_filename(tokens[1].clone());
    let new_directory = parse_filename(tokens[2].clone());
    return Ok(Command::RenameDirectory {
        old_directory,
        new_directory,
    });
}

// File Management Commands

fn parse_make_file(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() != 2 {
        return Err(format!("Expected {}", "MKFILE <filename>".red()));
    }
    let filename = parse_filename(tokens[1].clone());
    return Ok(Command::MakeFile { filename });
}

fn parse_remove_file(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() != 2 {
        return Err(format!("Expected {}", "RMFILE <filename>".red()));
    }
    let filename = parse_filename(tokens[1].clone());
    return Ok(Command::RemoveFile { filename });
}

fn parse_rename_file(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() != 3 {
        return Err(format!(
            "Expected {}",
            "RENFILE <old_filename> <new_filename>".red()
        ));
    }
    let old_filename = parse_filename(tokens[1].clone());
    let new_filename = parse_filename(tokens[2].clone());
    return Ok(Command::RenameFile {
        old_filename,
        new_filename,
    });
}

fn parse_change_drive(tokens: &Vec<String>) -> Result<Command, String> {
    if tokens.len() != 2 {
        return Err(format!("Expected {}", "CD <drive>".red()));
    }
    return Ok(Command::ChangeDrive {
        drive: tokens[1].to_uppercase(),
    });
}
fn parse_select(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() < 4 {
        return Err("SELECT requires: SELECT \"Filename\" FROM \"Directory\""
            .red()
            .to_string());
    }

    Ok(Command::Select {
        filename: parse_filename(tokens[1].clone()),
        directory: tokens[3].clone(),
    })
}

fn parse_view(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() == 1 && tokens[0].to_uppercase() == "VS" {
        return Ok(Command::ViewState);
    }
    if tokens.len() != 2 {
        return Err("VIEW requires: STATE keyword".red().to_string());
    }
    if tokens[1].to_uppercase() != "STATE" {
        return Err("VIEW requires: STATE keyword".red().to_string());
    }
    if tokens[0].to_uppercase() == "VIEW" && tokens[1].to_uppercase() == "STATE" {
        return Ok(Command::ViewState);
    }

    Err(
        "Expected VIEW STATE or VS\nType LC to viewa list of available commands."
            .red()
            .to_string(),
    )
}

fn parse_meta_state(tokens: &[String]) -> Result<Command, String> {
    return if tokens.len() < 2 && tokens[0].to_uppercase() == "MS" {
        Ok(Command::MetaState)
    } else if tokens.len() == 2 && tokens[0].to_uppercase() == "META" {
        if tokens[1].to_uppercase() != "STATE" {
            Err("META requires: STATE keyword".red().to_string())
        } else {
            Ok(Command::MetaState)
        }
    } else {
        Err("Unknown META or STATE keyword.".red().to_string())
    };
}

fn parse_drop_state(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() == 1 && tokens[0].to_uppercase() == "DS" {
        return Ok(Command::ClearState);
    }
    if tokens.len() == 2
        && tokens[0].to_uppercase() == "DROP"
        && tokens[1].to_uppercase() == "STATE"
    {
        return Ok(Command::ClearState);
    }
    Err(
        "Expected DROP STATE or DS. Type LC to view a list available commands."
            .red()
            .to_string(),
    )
}



fn parse_filename(token: String) -> String {
    // Handle quoted strings
    if (token.starts_with('"') && token.ends_with('"'))
        || (token.starts_with('\'') && token.ends_with('\''))
    {
        return token[1..token.len() - 1].to_string();
    }
    token.replace("\\ ", " ")
}

fn parse_fav(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() < 2 {
        return Err("FAV requires a subcommand (VIEW, SET, RM)"
            .red()
            .to_string());
    }

    match tokens[1].to_uppercase().as_str() {
        "VIEW" => Ok(Command::FavView),
        "SET" => {
            if tokens.len() >= 3 && tokens[2].to_uppercase() == "STATE" {
                Ok(Command::FavSet)
            } else {
                Err("Expected FAV SET STATE".red().to_string())
            }
        }
        "RM" => {
            if tokens.len() < 3 {
                return Err("FAV RM requires an index".red().to_string());
            }
            Ok(Command::FavRm {
                index: match tokens[2].parse::<usize>() {
                    Ok(idx) => idx,
                    Err(_) => {
                        crate::cprintln!("{}: Index out of bounds!", "ERROR".red());
                        return Ok(Command::Unknown {
                            command: "Invalid Fav Index".to_string(),
                            args: vec![],
                        });
                    }
                },
            })
        }
        _ => Err(
            "Unknown FAV subcommand. Type LC to view a list of available commands."
                .red()
                .to_string(),
        ),
    }
}

fn parse_run(tokens: &[String]) -> Result<Command, String> {
    // Pattern Matching based on Length of Tokens and First Token
    match (tokens.len(), tokens[0].to_uppercase().as_str()) {
        (2, "RUN") => match tokens[1].to_uppercase().as_str() {
            "STATE" => Ok(Command::RunState),
            _ => Err("Expected RUN STATE or RF".red().to_string()),
        },
        (3, "RUN") => match tokens[1].to_uppercase().as_str() {
            "FAV" => Ok(Command::RunFav {
                index: match tokens[2].parse::<usize>() {
                    Ok(idx) => idx,
                    Err(_) => {
                        crate::cprintln!("ERROR: Index out of bounds!");
                        return Ok(Command::Unknown {
                            command: "Invalid Fav Index".to_string(),
                            args: vec![],
                        });
                    }
                },
            }),
            _ => Err("Expected RUN FAV <index> or RF <index>".red().to_string()),
        },
        (1, "RS") => Ok(Command::RunState),
        (2, "RF") => Ok(Command::RunFav {
            index: match tokens[1].parse::<usize>() {
                Ok(idx) => idx,
                Err(_) => {
                    crate::cprintln!("ERROR: Invalid FAV index!");
                    return Ok(Command::Unknown {
                        command: tokens[0].clone(),
                        args: tokens[1..].to_vec(),
                    });
                }
            },
        }),
        _ => Err(
            "Invalid RUN Command. Type LC to view a list of available commands."
                .red()
                .to_string(),
        ),
    }
}

fn parse_unknown(tokens: &[String]) -> Result<Command, String> {
    return Ok(Command::Unknown {
        command: tokens[0].clone(),
        args: tokens[1..].to_vec(),
    });
}
