// use std::fmt::format;

use colored::Colorize;

use crate::search::SearchEngine;

#[derive(Debug)]
pub enum Command {
    ListCommands,
    Select {
        filename: String,
        directory: String,
    },
    ViewState,
    ClearState,
    RunState,
    MetaState,
    FindExact {
        filename: String,
    },
    Search {
        engine: SearchEngine,
        filename: String,
    },
    FavView,
    FavSet,
    FavRm {
        index: usize,
    },
    RunFav {
        index: usize,
    },
    Unknown {
        command: String,
    },
    DodgeDirectory,
    WatchDirectory {
        directory: String,
    },
    ListDirectory,
    ChangeDrive {
        drive: String,
    },
    ClearScreen,
    Exit,
}

pub fn parse_command(input: &str) -> Result<Command, String> {
    let tokens = tokenize(input)?;

    return match tokens[0].to_uppercase().as_str() {
        // Meta Commands
        "LC" => Ok(Command::ListCommands),
        "EXIT" | "/E" => Ok(Command::Exit),
        "CLS" | "/C" => Ok(Command::ClearScreen),
        // Directory Commands
        "DD" => Ok(Command::DodgeDirectory),
        "WD" => parse_watch_directory(&tokens),
        "LD" => parse_list_directory(&tokens),
        "CD" => parse_change_drive(&tokens),
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
        "FIND" | "FE" => parse_find_exact(&tokens),
        "SEARCH" | "S" => parse_search(&tokens),
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
    if tokens.len() > 1 {
        return Err("Expected no arguments with LD Command".red().to_string());
    }
    return Ok(Command::ListDirectory);
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

fn parse_find_exact(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() == 2 && tokens[0].to_uppercase() == "FE" {
        return Ok(Command::FindExact {
            filename: parse_filename(tokens[1].clone()),
        });
    }

    if tokens.len() == 3
        && tokens[0].to_uppercase() == "FIND"
        && tokens[1].to_uppercase() == "EXACT"
    {
        return Ok(Command::FindExact {
            filename: parse_filename(tokens[2].clone()),
        });
    }
    Err(
        "Expected FIND EXACT or FE \"Filename\", Type LC to view a list of available commands."
            .red()
            .to_string(),
    )
}

fn parse_search(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() != 3 {
        return Err("SEARCH Expected three arguments!".red().to_string());
    }
    if tokens[0].to_lowercase() == "search" || tokens[0].to_lowercase() == "s" {
        return match tokens[1].clone().to_lowercase().as_str() {
            "google" | "g" => Ok(Command::Search {
                engine: SearchEngine::Google,
                filename: parse_filename(tokens[2].clone()),
            }),
            "ddg" | "d" => Ok(Command::Search {
                engine: SearchEngine::DuckDuckGo,
                filename: parse_filename(tokens[2].clone()),
            }),
            "chatgpt" | "c" => Ok(Command::Search {
                engine: SearchEngine::ChatGPT,
                filename: parse_filename(tokens[2].clone()),
            }),
            "perplexity" | "p" => Ok(Command::Search {
                engine: SearchEngine::Perplexity,
                filename: parse_filename(tokens[2].clone()),
            }),
            &_ => {
                println!(
                    "{} > Unknown search engine: {}",
                    "ERROR".red(),
                    tokens[1].to_string()
                );
                Ok(Command::Unknown {
                    command: parse_filename(tokens[1].clone()),
                })
            }
        };
    }
    Err(
        "Invalid SEARCH Command. Run LIST COMMANDS or LC to view available commands."
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
                        println!("{}: Index out of bounds!", "ERROR".red());
                        return Ok(Command::Unknown {
                            command: "Invalid Fav Index".to_string(),
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
                        println!("ERROR: Index out of bounds!");
                        return Ok(Command::Unknown {
                            command: "Invalid Fav Index".to_string(),
                        });
                    }
                },
            }),
            _ => Err("Expected RUN FAV <index> or RF <index>".red().to_string()),
        },
        (1, "RS") => Ok(Command::RunState),
        (2, "RF") => Ok(Command::RunFav {
            index: tokens[1]
                .parse::<usize>()
                .expect("Invalid FAV index".red().to_string().as_str()),
        }),
        _ => Err(
            "Invalid RUN Command. Type LC to view a list of available commands."
                .red()
                .to_string(),
        ),
    }
}

fn parse_unknown(tokens: &[String]) -> Result<Command, String> {
    println!(
        "Unknown command. Type {} to view a list of available commands.",
        "LC".yellow().to_string()
    );
    return Ok(Command::Unknown {
        command: tokens[0].to_uppercase(),
    });
}
