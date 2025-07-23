#[derive(Debug)]
pub enum Command {
    Select { filename: String, directory: String },
    ViewState,
    ClearState,
    RunState,
    FindExact { filename: String },
    FavView,
    FavSet,
    FavRm { filename: String },
    RunFav { index: usize },
    Unknown { command: String },
}

pub fn parse_command(input: &str) -> Result<Command, String> {
    let tokens = tokenize(input)?;

    if tokens.is_empty() {
        return Err("Emp".to_string());
    }

    match tokens[0].to_uppercase().as_str() {
        "SELECT" => parse_select(&tokens),
        "VIEW" => parse_view(&tokens),
        "CLEAR" => clear_view(&tokens),
        "RUN" => match tokens[1].to_uppercase().as_str() {
            "STATE" => parse_run_state(&tokens),
            "FAV" => parse_run_fav(&tokens),
            _ => Ok(Command::Unknown {
                command: tokens[1].clone(),
            }),
        },
        "FIND" => parse_find_exact(&tokens),
        "FAV" => parse_fav(&tokens),
        _ => Ok(Command::Unknown {
            command: tokens[0].to_uppercase().clone(),
        }),
    };

    return Ok(Command::Unknown {
        command: input.to_string(),
    });
}

/// Splits an input string into tokens, handling quoted strings as single tokens
/// Returns a Result containing either a vector of tokens or an error message
fn tokenize(input: &str) -> Result<Vec<String>, String> {
    // Initialize collections to store tokens and current token being built
    let mut tokens = Vec::new();
    let mut current_token = String::new();

    // Track whether we're currently inside a quoted string
    let mut in_quotes = false;

    // Create peekable iterator over input characters
    let mut chars = input.chars().peekable();
    // For example, in a tokenizer, you might want to keep consuming characters until you
    // hit a whitespace or special character.
    // The peek() method lets you check if the next character is a whitespace
    // or special character before advancing the iterator.

    // Process each character in the input string
    while let Some(ch) = chars.next() {
        match ch {
            // Toggle quote state when encountering quote marks
            '"' => {
                in_quotes = !in_quotes;
            }
            // Handle whitespace characters
            ' ' | '\t' => {
                if in_quotes {
                    // Inside quotes, spaces are part of the token
                    current_token.push(ch);
                } else if !current_token.is_empty() {
                    // Outside quotes, spaces separate tokens
                    tokens.push(current_token.trim().to_string());
                    current_token.clear();
                }
            }
            // Add all other characters to current token
            _ => {
                // Regular characters are added to the current token
                current_token.push(ch);
            }
        }
    }

    // Handle any remaining token after processing all characters
    if !current_token.is_empty() {
        tokens.push(current_token.trim().to_string());
    }

    // Return error if quotes weren't properly closed
    if in_quotes {
        return Err("Unclosed quotes in command.".to_string());
    }

    // Return the collected tokens
    return Ok(tokens);
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
        filename: tokens[1].clone(),
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

    Err("Expected VIEW STATE or VS".to_string())
}

fn clear_view(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() == 1 && tokens[0].to_uppercase() == "CV" {
        return Ok(Command::ClearState);
    }
    if tokens.len() == 2
        && tokens[0].to_uppercase() == "CLEAR"
        && tokens[1].to_uppercase() == "VIEW"
    {
        return Ok(Command::ClearState);
    }
    Err("Expected CLEAR VIEW or CV".to_string())
}

fn parse_run_state(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() == 1 && tokens[0].to_uppercase() == "RS" {
        return Ok(Command::RunState);
    }
    if tokens.len() == 2 && tokens[0].to_uppercase() == "RUN" && tokens[1].to_uppercase() == "STATE"
    {
        return Ok(Command::RunState);
    }
    Err("Expected RUN STATE or RS".to_string())
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
    Err("Expected FIND EXACT or FE".to_string())
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
        _ => Err("Unknown FAV subcommand".to_string()),
    }
}

fn parse_run_fav(tokens: &[String]) -> Result<Command, String> {
    if tokens.len() == 2 && tokens[0].to_uppercase() == "RF" {
        return Ok(Command::RunFav {
            index: tokens[1]
                .to_string()
                .parse::<usize>()
                .expect("Invalid FAV index"),
        });
    }
    if tokens.len() == 3 && tokens[0].to_uppercase() == "RUN" && tokens[1].to_uppercase() == "FAV" {
        return Ok(Command::RunFav {
            index: tokens[1]
                .to_string()
                .parse::<usize>()
                .expect("Invalid FAV index"),
        });
    }
    Err("Expected RUN FAV or RF".to_string())
}
