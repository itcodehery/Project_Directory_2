use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: String,
    #[allow(dead_code)]
    pub is_directory: bool,
    #[allow(dead_code)]
    pub size: u64,
    #[allow(dead_code)]
    pub modified: Option<SystemTime>,
}

#[derive(Debug)]
pub struct FileIndex {
    files: Vec<FileInfo>,
    directories: Vec<FileInfo>,
    last_scan: Option<SystemTime>,
    root_path: PathBuf,
}

impl FileIndex {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            files: Vec::new(),
            directories: Vec::new(),
            last_scan: None,
            root_path,
        }
    }

    pub fn scan_directory(&mut self) -> Result<(), std::io::Error> {
        self.files.clear();
        self.directories.clear();
        
        self.scan_recursive(&self.root_path.clone())?;
        self.last_scan = Some(SystemTime::now());
        
        // Sort for better search performance
        self.files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        self.directories.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        
        Ok(())
    }

    fn scan_recursive(&mut self, path: &Path) -> Result<(), std::io::Error> {
        self.scan_recursive_with_depth(path, 0)
    }

    fn scan_recursive_with_depth(&mut self, path: &Path, depth: usize) -> Result<(), std::io::Error> {
        // Limit recursion depth to 3 levels to reduce resource usage
        if depth > 3 {
            return Ok(());
        }

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Skip hidden files and directories (starting with .)
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') {
                        continue;
                    }
                    // Skip common large directories to save resources
                    if matches!(name, "node_modules" | "target" | ".git" | "dist" | "build" | "__pycache__") {
                        continue;
                    }
                }

                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let file_info = FileInfo {
                    path: path.clone(),
                    name: path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_default()
                        .to_string(),
                    #[allow(dead_code)]
                    is_directory: metadata.is_dir(),
                    #[allow(dead_code)]
                    size: metadata.len(),
                    #[allow(dead_code)]
                    modified: metadata.modified().ok(),
                };

                if metadata.is_dir() {
                    self.directories.push(file_info);
                    // Recursively scan subdirectories with depth limit
                    let _ = self.scan_recursive_with_depth(&path, depth + 1);
                } else {
                    self.files.push(file_info);
                }
            }
        }
        Ok(())
    }

    pub fn search_files(&self, query: &str, limit: Option<usize>) -> Vec<&FileInfo> {
        let query_lower = query.to_lowercase();
        let mut matches: Vec<_> = self.files
            .iter()
            .filter(|file| {
                file.name.to_lowercase().contains(&query_lower) ||
                file.path.to_string_lossy().to_lowercase().contains(&query_lower)
            })
            .collect();

        // Sort by relevance: exact matches first, then prefix matches, then contains
        matches.sort_by(|a, b| {
            let a_name_lower = a.name.to_lowercase();
            let b_name_lower = b.name.to_lowercase();
            
            let a_exact = a_name_lower == query_lower;
            let b_exact = b_name_lower == query_lower;
            
            if a_exact != b_exact {
                return b_exact.cmp(&a_exact);
            }
            
            let a_prefix = a_name_lower.starts_with(&query_lower);
            let b_prefix = b_name_lower.starts_with(&query_lower);
            
            if a_prefix != b_prefix {
                return b_prefix.cmp(&a_prefix);
            }
            
            // If both are prefix matches or both are contains matches, sort by name length
            a.name.len().cmp(&b.name.len())
        });

        if let Some(limit) = limit {
            matches.truncate(limit);
        }

        matches
    }

    pub fn search_directories(&self, query: &str, limit: Option<usize>) -> Vec<&FileInfo> {
        let query_lower = query.to_lowercase();
        let mut matches: Vec<_> = self.directories
            .iter()
            .filter(|dir| {
                dir.name.to_lowercase().contains(&query_lower) ||
                dir.path.to_string_lossy().to_lowercase().contains(&query_lower)
            })
            .collect();

        // Sort by relevance similar to files
        matches.sort_by(|a, b| {
            let a_name_lower = a.name.to_lowercase();
            let b_name_lower = b.name.to_lowercase();
            
            let a_exact = a_name_lower == query_lower;
            let b_exact = b_name_lower == query_lower;
            
            if a_exact != b_exact {
                return b_exact.cmp(&a_exact);
            }
            
            let a_prefix = a_name_lower.starts_with(&query_lower);
            let b_prefix = b_name_lower.starts_with(&query_lower);
            
            if a_prefix != b_prefix {
                return b_prefix.cmp(&a_prefix);
            }
            
            a.name.len().cmp(&b.name.len())
        });

        if let Some(limit) = limit {
            matches.truncate(limit);
        }

        matches
    }

    pub fn get_files_in_directory(&self, dir_path: &Path) -> Vec<&FileInfo> {
        self.files
            .iter()
            .filter(|file| {
                if let Some(parent) = file.path.parent() {
                    parent == dir_path
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn get_directories_in_directory(&self, dir_path: &Path) -> Vec<&FileInfo> {
        self.directories
            .iter()
            .filter(|dir| {
                if let Some(parent) = dir.path.parent() {
                    parent == dir_path
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn needs_refresh(&self, max_age: Duration) -> bool {
        match self.last_scan {
            Some(last_scan) => {
                SystemTime::now()
                    .duration_since(last_scan)
                    .unwrap_or(Duration::MAX) > max_age
            }
            None => true,
        }
    }

    pub fn get_file_count(&self) -> usize {
        self.files.len()
    }

    pub fn get_directory_count(&self) -> usize {
        self.directories.len()
    }

    pub fn update_root_path(&mut self, new_root: PathBuf) {
        self.root_path = new_root;
        self.files.clear();
        self.directories.clear();
        self.last_scan = None;
    }
}