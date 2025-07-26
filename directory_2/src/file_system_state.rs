use std::env;
use std::path::PathBuf;

#[derive(Debug)]
pub struct FileSystemState {
    state: Option<PathBuf>,
    current_path: PathBuf,
}

impl FileSystemState {
    pub fn new() -> Self {
        Self {
            state: None,
            current_path: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
        }
    }

    pub fn get_current_state(&self) -> &Option<PathBuf> {
        &self.state
    }

    pub fn get_current_path(&self) -> &PathBuf {
        &self.current_path
    }

    pub fn set_current_state(&mut self, new_path: PathBuf) {
        self.state = Some(new_path);
    }

    pub fn set_current_directory(&mut self, new_path: PathBuf) {
        self.current_path = new_path;
    }

    pub fn clear_state(&mut self) {
        self.state = None;
    }
}