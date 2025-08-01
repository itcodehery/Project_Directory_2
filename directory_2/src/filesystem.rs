// use std::fs::File;
// use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{fs, io};

// Check if a given path exists and is valid
pub fn path_exists(path: &Path) -> bool {
    path.exists()
}

// Verify if a path is a file (and not a directory)
pub fn is_file(path: &Path) -> bool {
    path.is_file()
}

// Verify if a path is a directory
pub fn is_dir(path: &Path) -> bool {
    path.is_dir()
}

// Verify if a path is readable
// pub fn is_readable(path: &Path) -> bool {
//     fs::metadata(path).is_ok()
// }

// Convert relative paths to absolute paths based on a given directory
pub fn resolve_path(relative_path: &Path, base_directory: &Path) -> PathBuf {
    if relative_path.is_absolute() {
        // Already absolute, return as-is
        relative_path.to_path_buf()
    } else {
        // Join with base directory to make it absolute
        base_directory.join(relative_path)
    }
}

// Handle special cases like double "." (parent directory) and "." (current directory)
// fn normalize_path(path: &Path) -> Result<PathBuf, io::Error> {
//     // canonicalize() resolves "..", ".", symbolic links, and normalizes the path
//     fs::canonicalize(path)
// }

// Normalize paths (handle multiple slashes, resolve symbolic links if needed)
// pub fn resolve_and_normalize(
//     relative_path: &Path,
//     base_directory: &Path,
// ) -> Result<PathBuf, io::Error> {
//     let resolved = resolve_path(relative_path, base_directory);
//     normalize_path(&resolved)
// }

pub fn is_executable(path: &PathBuf) -> bool {
    if !is_file(path) {
        return false;
    }

    {
        // On Windows, check file extension
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            matches!(ext.as_str(), "exe" | "bat" | "cmd" | "com" | "scr" | "msi")
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct FileInfo {
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub is_readonly: bool,
}

pub fn get_file_metadata(path: &Path) -> Result<FileInfo, io::Error> {
    let metadata = fs::metadata(path)?;

    Ok(FileInfo {
        size: metadata.len(),
        modified: metadata.modified().ok(),
        is_readonly: metadata.permissions().readonly(),
    })
}

// Directory Functions
// pub fn list_dir_contents(path: &Path) -> io::Result<Vec<DirEntry>> {
//     if !path.is_dir() {
//         return Err(io::Error::new(
//             io::ErrorKind::InvalidInput,
//             "Not a directory",
//         ));
//     }

//     let entries = fs::read_dir(path)?.filter_map(|entry| entry.ok()).collect();

//     Ok(entries)
// }

pub fn get_directory_without_parent(path: &Path) -> String {
    let parent = path.parent().unwrap();
    let path = path.to_string_lossy();
    let path = path.replace(parent.to_str().unwrap(), "");
    return path.replace("\\", "");
}

// pub fn is_dir_the_root(path: &Path) -> bool {
//     path.parent().is_none()
// }

// Checks if a directory is empty.
// pub fn is_dir_empty(path: &Path) -> io::Result<bool> {
//     if !path.is_dir() {
//         return Err(io::Error::new(
//             io::ErrorKind::InvalidInput,
//             "Not a directory",
//         ));
//     }

//     let mut entries = fs::read_dir(path)?;
//     Ok(entries.next().is_none())
// }

// pub fn is_dir_readable(path: &Path) -> bool {
//     fs::read_dir(path).is_ok()
// }

// Checks if a directory is writable by trying to create and delete a temporary file.
// pub fn is_dir_writable(path: &Path) -> bool {
//     if !path.is_dir() {
//         return false;
//     }

//     let filename = format!(
//         ".tmp_write_check_{}",
//         SystemTime::now()
//             .duration_since(UNIX_EPOCH)
//             .unwrap_or_default()
//             .as_nanos()
//     );

//     let temp_path = path.join(filename);

//     // Try writing and deleting
//     match File::create(&temp_path) {
//         Ok(mut file) => {
//             let write_result = writeln!(file, "test");
//             let _ = file.flush();
//             let _ = fs::remove_file(&temp_path);

//             write_result.is_ok()
//         }
//         Err(_) => false,
//     }
// }

// Recursively searches for files with the exact given name within the directory tree.
// Returns a list of matching file paths.
// pub fn find_file_by_name(root: &Path, target_name: &str) -> io::Result<Vec<PathBuf>> {
//     let mut matches = Vec::new();

//     if !root.is_dir() {
//         return Err(io::Error::new(
//             io::ErrorKind::InvalidInput,
//             "Root path is not a directory",
//         ));
//     }

//     for entry in fs::read_dir(root)? {
//         let entry = entry?;
//         let path = entry.path();

//         if path.is_dir() {
//             // Recursively search in subdirectories
//             if let Ok(sub_matches) = find_file_by_name(&path, target_name) {
//                 matches.extend(sub_matches);
//             }
//         } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
//             if name == target_name {
//                 matches.push(path);
//             }
//         }
//     }

//     Ok(matches)
// }

// fn find_file_by_name_with_filter<F>(root: &Path, filter: F) -> io::Result<Vec<PathBuf>>
// where
//     F: Fn(&str) -> bool + Copy,
// {
//     let mut matches = Vec::new();

//     if !root.is_dir() {
//         return Err(io::Error::new(
//             io::ErrorKind::InvalidInput,
//             "Root path is not a directory",
//         ));
//     }

//     for entry in fs::read_dir(root)? {
//         let entry = entry?;
//         let path = entry.path();

//         if path.is_dir() {
//             if let Ok(sub_matches) = find_file_by_name_with_filter(&path, filter) {
//                 matches.extend(sub_matches);
//             }
//         } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
//             if filter(name) {
//                 matches.push(path);
//             }
//         }
//     }

//     Ok(matches)
// }

// pub fn find_file_by_name_case_insensitive(
//     root: &Path,
//     target_name: &str,
// ) -> io::Result<Vec<PathBuf>> {
//     let lower_target = target_name.to_lowercase();

//     find_file_by_name_with_filter(root, |name| name.to_lowercase() == lower_target)
// }
