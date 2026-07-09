use colored::Colorize;

pub fn show_docs(command_name: Option<String>) -> Result<String, String> {
    if let Some(cmd) = command_name {
        match cmd.to_uppercase().as_str() {
            // Directory Commands
            "CD" => {
                crate::cprintln!("{}", "Command: CD".bright_cyan().bold());
                crate::cprintln!("Description: Changes the current working directory.");
                crate::cprintln!("Usage: CD <path>");
                crate::cprintln!("Examples:");
                crate::cprintln!("  CD /var/log");
                crate::cprintln!("  CD ..");
                crate::cprintln!("  CD ~");
            }
            "UP" => {
                crate::cprintln!("{}", "Command: UP".bright_cyan().bold());
                crate::cprintln!("Description: Moves one level up in the directory tree. Identical to 'CD ..'.");
                crate::cprintln!("Usage: UP");
            }
            "WD" => {
                crate::cprintln!("{}", "Command: WD".bright_cyan().bold());
                crate::cprintln!("Description: Watch Directory. Prints the path of the current working directory. Identical to 'pwd' on Unix.");
                crate::cprintln!("Usage: WD");
            }
            "LD" => {
                crate::cprintln!("{}", "Command: LD".bright_cyan().bold());
                crate::cprintln!("Description: List Directory. Displays all files and folders in the current directory.");
                crate::cprintln!("Usage: LD");
            }
            "DD" => {
                crate::cprintln!("{}", "Command: DD".bright_cyan().bold());
                crate::cprintln!("Description: Dodge Directory. Deletes the current directory and all of its contents, then moves up one level. USE WITH EXTREME CAUTION!");
                crate::cprintln!("Usage: DD");
            }
            "MKDIR" => {
                crate::cprintln!("{}", "Command: MKDIR".bright_cyan().bold());
                crate::cprintln!("Description: Creates a new directory in the current working directory.");
                crate::cprintln!("Usage: MKDIR <dirname>");
            }
            "RMDIR" => {
                crate::cprintln!("{}", "Command: RMDIR".bright_cyan().bold());
                crate::cprintln!("Description: Removes an empty directory.");
                crate::cprintln!("Usage: RMDIR <dirname>");
            }
            "TOUCH" => {
                crate::cprintln!("{}", "Command: TOUCH".bright_cyan().bold());
                crate::cprintln!("Description: Creates a new empty file.");
                crate::cprintln!("Usage: TOUCH <filename>");
            }
            "RM" => {
                crate::cprintln!("{}", "Command: RM".bright_cyan().bold());
                crate::cprintln!("Description: Removes a file.");
                crate::cprintln!("Usage: RM <filename>");
            }
            // State Commands
            "SV" | "SAVE STATE" => {
                crate::cprintln!("{}", "Command: SV / SAVE STATE".yellow().bold());
                crate::cprintln!("Description: Saves the current working directory to a temporary state slot. You can jump back to this slot later using RS (Run State).");
                crate::cprintln!("Usage: SV");
            }
            "LS" | "LOAD STATE" => {
                crate::cprintln!("{}", "Command: LS / LOAD STATE".yellow().bold());
                crate::cprintln!("Description: Shows the directory currently saved in the state slot.");
                crate::cprintln!("Usage: LS");
            }
            "DS" | "DROP STATE" => {
                crate::cprintln!("{}", "Command: DS / DROP STATE".yellow().bold());
                crate::cprintln!("Description: Clears the temporary state slot.");
                crate::cprintln!("Usage: DS");
            }
            "RS" | "RUN STATE" => {
                crate::cprintln!("{}", "Command: RS / RUN STATE".yellow().bold());
                crate::cprintln!("Description: Teleports you instantly to the directory saved in the temporary state slot.");
                crate::cprintln!("Usage: RS");
            }
            // Favorites Commands
            "FAV" => {
                crate::cprintln!("{}", "Command: FAV (Favorites System)".green().bold());
                crate::cprintln!("Description: DIR2 includes a permanent favorites system for directories you frequently visit.");
                crate::cprintln!("Subcommands:");
                crate::cprintln!("  FAV ADD <name> : Saves the current directory to favorites.");
                crate::cprintln!("  FAV LS         : Lists all favorite directories along with their index IDs.");
                crate::cprintln!("  FAV RM <id>    : Removes a favorite by its index ID.");
                crate::cprintln!("  RF <id>        : Teleports to a favorite directory by its index ID.");
            }
            "RF" | "RUN FAV" => {
                crate::cprintln!("{}", "Command: RF / RUN FAV".green().bold());
                crate::cprintln!("Description: Instantly jumps to a bookmarked directory using its ID from 'FAV LS'.");
                crate::cprintln!("Usage: RF <id>");
                crate::cprintln!("Example: RF 1");
            }
            // Search Commands
            "S" | "SEARCH" => {
                crate::cprintln!("{}", "Command: S / SEARCH".bright_magenta().bold());
                crate::cprintln!("Description: Opens your default web browser and searches the specified engine for your query.");
                crate::cprintln!("Supported Engines: google, ddg (DuckDuckGo), chatgpt, perplexity");
                crate::cprintln!("Usage: S <engine> <query>");
                crate::cprintln!("Example: S google \"how to write a rust macro\"");
            }
            // Environment Commands
            "EXPORT" => {
                crate::cprintln!("{}", "Command: EXPORT".magenta().bold());
                crate::cprintln!("Description: Sets an environment variable for the current DIR2 session. The variable will be substituted if referenced as $VAR or ${{VAR}}.");
                crate::cprintln!("Usage: EXPORT <VAR>=<value>");
            }
            "UNSET" => {
                crate::cprintln!("{}", "Command: UNSET".magenta().bold());
                crate::cprintln!("Description: Removes an environment variable.");
                crate::cprintln!("Usage: UNSET <VAR>");
            }
            "ENV" => {
                crate::cprintln!("{}", "Command: ENV".magenta().bold());
                crate::cprintln!("Description: Lists all active environment variables.");
                crate::cprintln!("Usage: ENV");
            }
            "ECHO" => {
                crate::cprintln!("{}", "Command: ECHO".magenta().bold());
                crate::cprintln!("Description: Prints text to the terminal. Highly useful for testing variable substitution.");
                crate::cprintln!("Usage: ECHO <text>");
            }
            // Alias Commands
            "ALIAS" => {
                crate::cprintln!("{}", "Command: ALIAS".bright_green().bold());
                crate::cprintln!("Description: Creates a custom shortcut for a longer command.");
                crate::cprintln!("Usage: ALIAS <name>='<command>'");
                crate::cprintln!("Example: ALIAS ll='ls -la'");
            }
            "UNALIAS" => {
                crate::cprintln!("{}", "Command: UNALIAS".bright_green().bold());
                crate::cprintln!("Description: Removes a previously created alias.");
                crate::cprintln!("Usage: UNALIAS <name>");
            }
            "ALIASES" => {
                crate::cprintln!("{}", "Command: ALIASES".bright_green().bold());
                crate::cprintln!("Description: Lists all active aliases.");
                crate::cprintln!("Usage: ALIASES");
            }
            // TUI Config
            "TUIADD" => {
                crate::cprintln!("{}", "Command: TUIADD".blue().bold());
                crate::cprintln!("Description: Adds a command to the interactive whitelist. Commands on this list will cleanly suspend the TUI so they can run fullscreen natively.");
                crate::cprintln!("Usage: TUIADD <command>");
            }
            "TUIRM" => {
                crate::cprintln!("{}", "Command: TUIRM".blue().bold());
                crate::cprintln!("Description: Removes a command from the interactive whitelist.");
                crate::cprintln!("Usage: TUIRM <command>");
            }
            "TUILS" => {
                crate::cprintln!("{}", "Command: TUILS".blue().bold());
                crate::cprintln!("Description: Lists all commands that are whitelisted to run interactively outside the TUI.");
                crate::cprintln!("Usage: TUILS");
            }
            // Meta Commands
            "LC" => {
                crate::cprintln!("{}", "Command: LC (List Commands)".bright_blue().bold());
                crate::cprintln!("Description: Prints a quick cheatsheet of all built-in commands.");
                crate::cprintln!("Usage: LC");
            }
            "CLS" | "CLEAR" | "/C" => {
                crate::cprintln!("{}", "Command: CLS / CLEAR".bright_blue().bold());
                crate::cprintln!("Description: Clears the screen of all past output logs, while spawning a clickable [History] button in the header so you can restore them if needed.");
                crate::cprintln!("Usage: CLS");
            }
            "DOCS" | "MAN" => {
                crate::cprintln!("{}", "Command: DOCS / MAN".bright_blue().bold());
                crate::cprintln!("Description: You are looking at it! Provides detailed manual pages for every command in DIR2.");
                crate::cprintln!("Usage: DOCS <command>");
            }
            "EXIT" | "QUIT" | "/Q" => {
                crate::cprintln!("{}", "Command: EXIT".bright_blue().bold());
                crate::cprintln!("Description: Gracefully shuts down the DIR2 shell.");
                crate::cprintln!("Usage: EXIT");
            }
            _ => {
                crate::cprintln!("{} No documentation found for '{}'.", "ERROR:".red(), cmd);
                crate::cprintln!("Type {} for a general overview, or {} for a quick command list.", "DOCS".yellow(), "LC".yellow());
            }
        }
    } else {
        // General Overview
        crate::cprintln!("{}", "=========================================================".bright_cyan());
        crate::cprintln!("{}", "              DIR2 COMPREHENSIVE MANUAL                  ".bright_cyan().bold());
        crate::cprintln!("{}", "=========================================================".bright_cyan());
        crate::cprintln!("\nWelcome to DIR2! DIR2 is an ultra-fast, modern shell replacement written in Rust.");
        crate::cprintln!("It blends standard shell execution with a beautiful Ratatui-powered Terminal User Interface.");
        crate::cprintln!("\n{} Native Execution", "[*]".bright_green());
        crate::cprintln!("DIR2 automatically acts as a standard shell! If you type a command that is not built-in (like 'git status' or 'cargo build'), DIR2 executes it on your OS seamlessly and captures the output.");
        crate::cprintln!("It even detects full-screen apps (like 'vim' or 'htop') and safely suspends the UI to run them!");
        
        crate::cprintln!("\n{} Startup Scripts", "[*]".bright_green());
        crate::cprintln!("You can place commands, aliases, and exports inside {} in your home directory.", "~/.dir2rc".yellow());
        crate::cprintln!("DIR2 will automatically load and run this script every time you start it up.");
        
        crate::cprintln!("\n{} Built-in Categories", "[*]".bright_green());
        crate::cprintln!("  {} CD, UP, WD, LD, DD, MKDIR, RMDIR, TOUCH, RM", "Directory:".cyan());
        crate::cprintln!("  {} S", "Search:".cyan());
        crate::cprintln!("  {} FAV ADD, FAV LS, FAV RM, RF", "Favorites:".green());
        crate::cprintln!("  {} SV, LS, DS, RS", "State:".yellow());
        crate::cprintln!("  {} EXPORT, UNSET, ENV, ECHO", "Environment:".magenta());
        crate::cprintln!("  {} ALIAS, UNALIAS, ALIASES", "Alias:".bright_green());
        crate::cprintln!("  {} TUIADD, TUIRM, TUILS", "TUI Config:".blue());
        crate::cprintln!("  {} LC, DOCS, CLS, EXIT", "Meta:".bright_blue());
        
        crate::cprintln!("\nFor detailed help on a specific command, type: {} {}", "DOCS".yellow(), "<command>".white());
        crate::cprintln!("Example: {}", "DOCS FAV".yellow());
    }
    
    Ok(String::new())
}
