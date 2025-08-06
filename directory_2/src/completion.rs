use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::Hinter;
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::{Context};
use rustyline::Helper;
use std::borrow::Cow::{self, Borrowed, Owned};
use std::path::{Path, PathBuf};
use std::time::Duration;
use crate::indexing::FileIndex;

pub struct Dir2Helper {
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    current_directory: PathBuf,
    file_index: FileIndex,
}

impl Dir2Helper {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let mut file_index = FileIndex::new(current_dir.clone());
        
        // Initial scan (ignore errors for now)
        let _ = file_index.scan_directory();
        
        Dir2Helper {
            highlighter: MatchingBracketHighlighter::new(),
            validator: MatchingBracketValidator::new(),
            current_directory: current_dir,
            file_index,
        }
    }

    pub fn update_current_directory(&mut self, path: PathBuf) {
        self.current_directory = path.clone();
        self.file_index.update_root_path(path);
        // Trigger a rescan
        let _ = self.file_index.scan_directory();
    }

    pub fn refresh_index_if_needed(&mut self) {
        // Refresh index every 5 minutes instead of 30 seconds to reduce resource usage
        if self.file_index.needs_refresh(Duration::from_secs(300)) {
            let _ = self.file_index.scan_directory();
        }
    }

    pub fn get_index_stats(&self) -> (usize, usize) {
        (self.file_index.get_file_count(), self.file_index.get_directory_count())
    }

    fn get_commands() -> Vec<&'static str> {
        vec![
            // Meta Commands
            "LC", "EXIT", "/E", "CLS", "/C", "INDEX",
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

    pub fn get_directories_in_current_path(&self, partial_name: &str) -> Vec<String> {
        // First try current directory only
        let current_dir_matches = self.file_index
            .get_directories_in_directory(&self.current_directory)
            .into_iter()
            .filter(|dir| dir.name.to_lowercase().starts_with(&partial_name.to_lowercase()))
            .map(|dir| dir.name.clone())
            .collect::<Vec<_>>();

        if !current_dir_matches.is_empty() {
            current_dir_matches
        } else {
            // If no matches in current directory, search globally
            self.file_index
                .search_directories(partial_name, Some(10))
                .into_iter()
                .map(|dir| {
                    // Show relative path if it's not in current directory
                    if let Ok(rel_path) = dir.path.strip_prefix(&self.current_directory) {
                        rel_path.to_string_lossy().to_string()
                    } else {
                        dir.name.clone()
                    }
                })
                .collect()
        }
    }

    pub fn get_files_in_current_path(&self, partial_name: &str) -> Vec<String> {
        // First try current directory only
        let current_dir_matches = self.file_index
            .get_files_in_directory(&self.current_directory)
            .into_iter()
            .filter(|file| file.name.to_lowercase().starts_with(&partial_name.to_lowercase()))
            .map(|file| file.name.clone())
            .collect::<Vec<_>>();

        if !current_dir_matches.is_empty() {
            current_dir_matches
        } else {
            // If no matches in current directory, search globally
            self.file_index
                .search_files(partial_name, Some(15))
                .into_iter()
                .map(|file| {
                    // Show relative path if it's not in current directory
                    if let Ok(rel_path) = file.path.strip_prefix(&self.current_directory) {
                        rel_path.to_string_lossy().to_string()
                    } else {
                        file.name.clone()
                    }
                })
                .collect()
        }
    }

    fn get_global_file_suggestions(&self, partial_name: &str) -> Vec<String> {
        self.file_index
            .search_files(partial_name, Some(20))
            .into_iter()
            .map(|file| {
                // For global suggestions, show relative path for context
                if let Ok(rel_path) = file.path.strip_prefix(&self.current_directory) {
                    if rel_path.parent().is_some() && rel_path.parent().unwrap() != Path::new("") {
                        format!("{} ({})", file.name, rel_path.parent().unwrap().display())
                    } else {
                        file.name.clone()
                    }
                } else {
                    format!("{} ({})", file.name, file.path.parent().unwrap_or(&file.path).display())
                }
            })
            .collect()
    }

    fn detect_completion_context(&self, line: &str, pos: usize) -> CompletionContext {
        let words: Vec<&str> = line.split_whitespace().collect();
        
        if words.is_empty() {
            return CompletionContext::Command;
        }

        let command = words[0].to_uppercase();
        
        match command.as_str() {
            "WD" => CompletionContext::Directory,
            "SELECT" => {
                if words.len() == 1 || (words.len() == 2 && pos <= line.len()) {
                    CompletionContext::File
                } else if words.len() >= 3 && words.get(2) == Some(&"FROM") {
                    CompletionContext::Directory
                } else {
                    CompletionContext::File
                }
            }
            "CD" => CompletionContext::Drive,
            "FIND" | "FE" | "SEARCH" | "S" => CompletionContext::GlobalFile,
            "RF" | "RUN" => CompletionContext::File,
            _ => CompletionContext::Command,
        }
    }
}

#[derive(Debug, PartialEq)]
enum CompletionContext {
    Command,
    Directory,
    File,
    Drive,
    GlobalFile,
}

impl Completer for Dir2Helper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        // Note: We can't mutate self here due to the trait signature, so we can't refresh the index
        // The refresh should be called from the main loop periodically
        
        let (start, word) = extract_word(line, pos);
        let context = self.detect_completion_context(line, pos);
        
        match context {
            CompletionContext::Command => {
                // Complete command names
                let commands = Self::get_commands();
                let matches: Vec<Pair> = commands
                    .iter()
                    .filter(|cmd| cmd.to_uppercase().starts_with(&word.to_uppercase()))
                    .map(|cmd| Pair {
                        display: cmd.to_string(),
                        replacement: cmd.to_string(),
                    })
                    .collect();
                
                Ok((start, matches))
            }
            CompletionContext::Directory => {
                // Complete directory names from current directory first, then global
                let directories = self.get_directories_in_current_path(word);
                let matches: Vec<Pair> = directories
                    .into_iter()
                    .map(|dir| Pair {
                        display: format!("{}/", dir),
                        replacement: dir,
                    })
                    .collect();
                
                Ok((start, matches))
            }
            CompletionContext::File => {
                // Complete file names from current directory first, then global
                let files = self.get_files_in_current_path(word);
                let matches: Vec<Pair> = files
                    .into_iter()
                    .map(|file| Pair {
                        display: file.clone(),
                        replacement: file,
                    })
                    .collect();
                
                Ok((start, matches))
            }
            CompletionContext::GlobalFile => {
                // Global file search for FIND, SEARCH commands
                let files = self.get_global_file_suggestions(word);
                let matches: Vec<Pair> = files
                    .into_iter()
                    .map(|file| {
                        let actual_name = if file.contains(" (") {
                            file.split(" (").next().unwrap_or(&file).to_string()
                        } else {
                            file.clone()
                        };
                        Pair {
                            display: file,
                            replacement: actual_name,
                        }
                    })
                    .collect();
                
                Ok((start, matches))
            }
            CompletionContext::Drive => {
                // Complete drive letters (A-Z)
                let drives: Vec<String> = (b'A'..=b'Z')
                    .map(|c| (c as char).to_string())
                    .filter(|drive| drive.to_uppercase().starts_with(&word.to_uppercase()))
                    .collect();
                
                let matches: Vec<Pair> = drives
                    .into_iter()
                    .map(|drive| Pair {
                        display: format!("{}:\\", drive),
                        replacement: drive,
                    })
                    .collect();
                
                Ok((start, matches))
            }
        }
    }
}

impl Hinter for Dir2Helper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        // Only provide hints if we're at the end of the line
        if pos != line.len() {
            return None;
        }

        let (_, word) = extract_word(line, pos);
        let context = self.detect_completion_context(line, pos);
        
        // Only provide hints for certain contexts and if word is not empty
        if word.is_empty() {
            return None;
        }

        match context {
            CompletionContext::Directory => {
                // Get directory suggestions
                let directories = self.get_directories_in_current_path(word);
                if let Some(first_match) = directories.first() {
                    if first_match.to_lowercase().starts_with(&word.to_lowercase()) && first_match.len() > word.len() {
                        // Return the remaining part of the suggestion
                        return Some(first_match[word.len()..].to_string());
                    }
                }
            }
            CompletionContext::File => {
                // Get file suggestions
                let files = self.get_files_in_current_path(word);
                if let Some(first_match) = files.first() {
                    if first_match.to_lowercase().starts_with(&word.to_lowercase()) && first_match.len() > word.len() {
                        // Return the remaining part of the suggestion
                        return Some(first_match[word.len()..].to_string());
                    }
                }
            }
            CompletionContext::Command => {
                // Get command suggestions
                let commands = Self::get_commands();
                for cmd in commands {
                    if cmd.to_lowercase().starts_with(&word.to_lowercase()) && cmd.len() > word.len() {
                        return Some(cmd[word.len()..].to_string());
                    }
                }
            }
            CompletionContext::GlobalFile => {
                // Get global file suggestions
                let files = self.get_global_file_suggestions(word);
                if let Some(first_match) = files.first() {
                    // Extract just the filename part for the hint
                    let filename = if first_match.contains(" (") {
                        first_match.split(" (").next().unwrap_or(first_match)
                    } else {
                        first_match
                    };
                    if filename.to_lowercase().starts_with(&word.to_lowercase()) && filename.len() > word.len() {
                        return Some(filename[word.len()..].to_string());
                    }
                }
            }
            _ => {}
        }

        None
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
        // Style hints in dark gray (like PowerShell)
        Owned(format!("\x1b[90m{}\x1b[0m", hint))
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
