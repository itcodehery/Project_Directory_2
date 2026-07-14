use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::highlight::Highlighter;
use rustyline::validate::{Validator, ValidationResult, ValidationContext};
use rustyline::Helper;
use rustyline::Context;
use std::borrow::Cow;

pub struct Dir2Helper {
    hinter: HistoryHinter,
    completer: FilenameCompleter,
}

impl Dir2Helper {
    pub fn new() -> Self {
        Self {
            hinter: HistoryHinter {},
            completer: FilenameCompleter::new(),
        }
    }
}

impl Helper for Dir2Helper {}

impl Hinter for Dir2Helper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for Dir2Helper {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[90m{}\x1b[0m", hint))
    }
}

impl Validator for Dir2Helper {
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Completer for Dir2Helper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // First try to use the standard FilenameCompleter
        let (start, mut matches) = self.completer.complete(line, pos, ctx)?;
        
        // Autocomplete internal DIR2 commands for the first word
        if start == 0 {
            let internal_commands = vec![
                "CD ", "UP ", "WD ", "LD ", "DD ", "MKDIR ", "RMDIR ", "TOUCH ", "RM ",
                "S ", "FAV ", "RF ", "SV ", "LS ", "DS ", "RS ",
                "EXPORT ", "UNSET ", "ENV ", "ECHO ", "ALIAS ", "UNALIAS ", "ALIASES ",
                "JOBS ", "FG ", "KILL ", "SELECT ", "LC ", "CLS ", "DOCS ", "EXIT ",
                "PIPE "
            ];
            
            let word = &line[start..pos];
            let word_upper = word.to_uppercase();
            
            for cmd in internal_commands {
                if cmd.starts_with(&word_upper) {
                    matches.push(Pair {
                        display: cmd.to_string(),
                        replacement: cmd.to_string(),
                    });
                }
            }
        }
        
        Ok((start, matches))
    }
}
