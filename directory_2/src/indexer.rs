use crate::{file_system_state::FileSystemState, filesystem};

pub fn index_current_directory(file_system_state: &mut FileSystemState) {
    let files = filesystem::list_all_files_in_directory(&file_system_state.get_current_path());

    let mut index: Vec<String> = Vec::new();
    for file in files {
        index.push(file);
    }
    file_system_state.set_index(index);
}
