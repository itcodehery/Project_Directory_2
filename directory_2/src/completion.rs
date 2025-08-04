use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::{Context};
use rustyline::Helper;
use std::borrow::Cow::{self, Borrowed, Owned};

pub struct Dir2Helper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
}

impl Dir2Helper {
    pub fn new() -> Self {
        Dir2Helper {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            validator: MatchingBracketValidator::new(),
            hinter: HistoryHinter::new(),
        }
    }

    fn get_commands() -> Vec<&'static str> {
        vec![
            // Meta Commands
            "LC", "EXIT", "/E", "CLS", "/C",
            // Directory Commands
            "DD", "WD", "LD", "CD",
            // State Commands
            "SELECT", "VIEW", "VS", "DROP", "DS", "RUN", "RS", "META",
            // Favorite Commands
            "RF", "FAV",
            // Search Commands
            "FIND", "FE", "SEARCH", "S",
        ]
    }
}

impl Completer for Dir2Helper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let commands = Self::get_commands();
        
        // Get the current word being typed
        let (start, word) = extract_word(line, pos);
        
        // If we're at the beginning or after whitespace, complete commands
        if start == 0 || line[..start].trim().is_empty() {
            let matches: Vec<Pair> = commands
                .iter()
                .filter(|cmd| cmd.to_uppercase().starts_with(&word.to_uppercase()))
                .map(|cmd| Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                })
                .collect();
            
            Ok((start, matches))
        } else {
            // For other positions, fall back to filename completion
            self.completer.complete(line, pos, ctx)
        }
    }
}

impl Hinter for Dir2Helper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for Dir2Helper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[90m".to_owned() + hint + "\x1b[0m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
    }
}

impl Validator for Dir2Helper {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

impl Helper for Dir2Helper {}

// Helper function to extract the current word being typed
fn extract_word(line: &str, pos: usize) -> (usize, &str) {
    let line_chars: Vec<char> = line.chars().collect();
    
    // Find the start of the current word
    let mut start = pos;
    while start > 0 && !line_chars[start - 1].is_whitespace() {
        start -= 1;
    }
    
    // Find the end of the current word
    let mut end = pos;
    while end < line_chars.len() && !line_chars[end].is_whitespace() {
        end += 1;
    }
    
    let word = &line[start..end];
    (start, word)
}
