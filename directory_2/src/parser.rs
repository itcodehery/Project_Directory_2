#[derive(Debug)]
pub enum Command {
    ListCommands,
    Select { filename: String, directory: String },
    ViewState,
    ClearState,
    RunState,
    MetaState,
    FindExact { filename: String },
    FavView,
    FavSet,
    FavRm { filename: String },
    RunFav { index: usize },
    Unknown { command: String },
    DodgeDirectory,
    WatchDirectory { directory: String },
    ClearScreen,
    Exit,
}

pub fn parse_command(input: &str) -> Result<Command, String> {
    let tokens = tokenize(input)?;

    if tokens.is_empty() {
        return Err("Emp".to_string());
    }

    return match tokens[0].to_uppercase().as_str() {
        // Meta Commands
        "LC"=> Ok(Command::ListCommands),
        "EXIT" | "/E" => Ok(Command::Exit),
        "CLS" | "/C"=> Ok(Command::ClearScreen),
        // Directory Commands
        "DD" => Ok(Command::DodgeDirectory),
        "WD"=> parse_watch_directory(&tokens),
        // STATE Commands
        "SELECT" => parse_select(&tokens),
        "VIEW" | "VS" => parse_view(&tokens),
        "DROP" => parse_drop_state(&tokens),
        "RUN" | "RS" => parse_run(&tokens),
        "META" => parse_meta_state(&tokens),
        // Favorite Commands
        "RF" => parse_run(&tokens),
        "FAV" => parse_fav(&tokens),
        // Search Commands
        "FIND" | "FE" => parse_find_exact(&tokens),
        _ => Ok(Command::Unknown {
            command: tokens[0].to_uppercase().clone(),
        }),
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
        return Err("Unclosed quotes in command.".to_string());
    }

    return Ok(tokens);
}

fn parse_watch_directory(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() < 2 {
        return Err("Expected <directory> AFTER WD".to_string());
    }
    Ok(Command::WatchDirectory { directory: tokens[1].clone() })
}
fn parse_select(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() < 4 {
        return Err("SELECT requires: SELECT \"Filename\" FROM \"Directory\"".to_string());
    }

    // Check for FROM keyword
    if tokens[2].to_lowercase() != "from" {
        return Err("Expected FROM keyword after file name.".to_string());
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
        return Err("VIEW requires: STATE keyword".to_string());
    }
    if tokens[1].to_uppercase() != "STATE" {
        return Err("VIEW requires: STATE keyword".to_string());
    }
    if tokens[0].to_uppercase() == "VIEW" && tokens[1].to_uppercase() == "STATE" {
        return Ok(Command::ViewState);
    }

    Err(
        "Expected VIEW STATE or VS\nType LC or LIST COMMANDS to view available commands."
            .to_string(),
    )
}

fn parse_meta_state(tokens: &[String]) -> Result<Command, String> {
    return if tokens.len() < 2 && tokens[0].to_uppercase() == "MS" {
        Ok(Command::MetaState)
    } else if tokens.len() == 2 && tokens[0].to_uppercase() == "META" {
        if tokens[1].to_uppercase() != "STATE" {
            Err("META requires: STATE keyword".to_string())
        } else {
            Ok(Command::MetaState)
        }
    } else {
        Err("Unknown META or STATE keyword.".to_string())
    }
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
        "Expected DROP STATE or DS. Type LC or LIST COMMANDS to view available commands."
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
        "Expected FIND EXACT or FE \"Filename\", Type LC or LIST COMMANDS to view available commands."
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
        return Err("FAV requires a subcommand (VIEW, SET, RM)".to_string());
    }

    match tokens[1].to_uppercase().as_str() {
        "VIEW" => Ok(Command::FavView),
        "SET" => {
            if tokens.len() >= 3 && tokens[2].to_uppercase() == "STATE" {
                Ok(Command::FavSet)
            } else {
                Err("Expected FAV SET STATE".to_string())
            }
        }
        "RM" => {
            if tokens.len() < 3 {
                return Err("FAV RM requires a filename".to_string());
            }
            Ok(Command::FavRm {
                filename: tokens[2].clone(),
            })
        }
        _ => Err(
            "Unknown FAV subcommand. Type LC or LIST COMMANDS to view available commands."
                .to_string(),
        ),
    }
}

fn parse_run(tokens: &[String]) -> Result<Command, String> {
    // Pattern Matching based on Length of Tokens and First Token
    match (tokens.len(), tokens[0].to_uppercase().as_str()) {
        (2, "RUN") => match tokens[1].to_uppercase().as_str() {
            "STATE" => Ok(Command::RunState),
            _ => Err("Expected RUN STATE or RF".to_string()),
        },
        (3, "RUN") => match tokens[1].to_uppercase().as_str() {
            "FAV" => Ok(Command::RunFav {
                index: tokens[2].parse::<usize>().expect("Invalid FAV index"),
            }),
            _ => Err("Expected RUN FAV <index> or RF <index>".to_string()),
        },
        (1, "RS") => Ok(Command::RunState),
        (2, "RF") => Ok(Command::RunFav {
            index: tokens[1].parse::<usize>().expect("Invalid FAV index"),
        }),
        _ => Err(
            "Invalid RUN Command. Type LC or LIST COMMANDS to view available commands.".to_string(),
        ),
    }
}
