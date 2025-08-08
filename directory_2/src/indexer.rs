use std::path::Path;
use crate::{file_system_state::FileSystemState, filesystem};

pub fn index_current_directory(file_system_state: &mut FileSystemState) {
    let files = filesystem::list_all_contents_in_directory(&file_system_state.get_current_path());

    let mut index: Vec<String> = Vec::new();
    for file in files {
        // Extract just the filename from the full path
        if let Some(filename) = Path::new(&file).file_name() {
            if let Some(filename_str) = filename.to_str() {
                index.push(filename_str.to_string());
            }
        }
    }
    file_system_state.set_index(index);
}
