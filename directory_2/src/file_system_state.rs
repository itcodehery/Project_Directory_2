use std::env;
use std::path::PathBuf;
use crate::indexer::index_current_directory;

use std::collections::HashMap;

#[derive(Debug)]
pub struct FileSystemState {
    state: Option<PathBuf>,
    index: Vec<String>,
    current_path: PathBuf,
    pub aliases: HashMap<String, String>,
    pub interactive_commands: Vec<String>,
}

impl FileSystemState {
    pub fn new() -> Self {
        let current_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let default_interactive = vec![
            "vim", "nvim", "vi", "nano", "emacs", "lazygit", "bacon", "tty-clock", 
            "top", "htop", "btm", "less", "more", "man", "fzf", "bat", "k9s", 
            "ssh", "python", "node", "irb", "psql", "mysql", "sqlite3", "gdb", "cgdb"
        ].into_iter().map(|s| s.to_string()).collect();

        let mut fs_state = Self {
            state: None,
            index: Vec::new(),
            current_path,
            aliases: HashMap::new(),
            interactive_commands: default_interactive,
        };

        // Index the current directory immediately
        index_current_directory(&mut fs_state);
        fs_state
    }


    pub fn get_current_state(&self) -> &Option<PathBuf> {
        &self.state
    }

    pub fn get_current_path(&self) -> &PathBuf {
        &self.current_path
    }

    pub fn get_all_indexed(&self) -> &Vec<String> {
        &self.index
    }

    pub fn set_index(&mut self, index: Vec<String>) {
        self.index = index;
    }

    pub fn clear_index(&mut self) {
        self.index.clear();
    }

    pub fn set_current_state(&mut self, new_path: PathBuf) {
        self.state = Some(new_path);
    }

    pub fn set_current_directory(&mut self, new_path: PathBuf) -> Result<(), std::io::Error> {
        // Try to change the directory first
        std::env::set_current_dir(&new_path)?;

        // Only update internal state if the directory change succeeded
        self.current_path = new_path;
        self.clear_index();

        // Re-index the new directory
        crate::indexer::index_current_directory(self);

        Ok(())
    }

    pub fn clear_state(&mut self) {
        self.state = None;
    }

    pub fn expand_aliases(&self, cmd: &str) -> String {
        let first_word = cmd.split_whitespace().next().unwrap_or("").to_string();
        if let Some(alias_val) = self.aliases.get(&first_word) {
            cmd.replacen(&first_word, alias_val, 1)
        } else {
            cmd.to_string()
        }
    }
}
