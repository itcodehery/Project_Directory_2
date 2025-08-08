use crate::file_system_state::FileSystemState;

/// Returns a vector of all completions that match the given input prefix
///
/// # Arguments
/// * `file_system_state` - Mutable reference to the file system state
/// * `input` - The current input strings to match against
///
/// # Returns
/// * `Vec<String>` - Vector of all matching completions

pub fn completion_engine(file_system_state: &mut FileSystemState, input: &str) -> Vec<String> {
    crate::indexer::index_current_directory(file_system_state);
    let current_index = file_system_state.get_all_indexed();
    let mut completions = Vec::new();

    if input.is_empty() {
        return completions;
    }

    // Extract the last word (the part we want to autocomplete)
    let words: Vec<&str> = input.split_whitespace().collect();
    let last_word = if input.ends_with(' ') {
        "" // If input ends with space, we're starting a new word
    } else {
        words.last().unwrap_or(&"")
    };

    // Find completions for the last word only
    for item in current_index {
        if item.starts_with(last_word) {
            completions.push(item.clone());
        }
    }

    completions.sort();
    completions
}

/// Alternative function that provides the best single completion (for tab completion)
/// This maintains the original behavior for cases where you want auto-completion
///
/// # Arguments
/// * `file_system_state` - Mutable reference to the file system state
/// * `input` - Mutable reference to the input string to modify
///
/// # Returns
/// * `bool` - Returns true if a completion was found and applied
pub fn auto_complete_single(file_system_state: &mut FileSystemState, input: &mut String) -> bool {
    let current_index = file_system_state.get_all_indexed();

    for item in current_index {
        if item.starts_with(input.as_str()) {
            // Clear the input and replace with the completion
            input.clear();
            input.push_str(&item);
            return true;
        }
    }

    false
}

/// Gets the longest common prefix among all completions
/// Useful for partial completion when multiple matches exist
///
/// # Arguments
/// * `file_system_state` - Mutable reference to the file system state
/// * `input` - The current input string to match against
///
/// # Returns
/// * `String` - The longest common prefix among all matches
pub fn get_common_prefix(file_system_state: &mut FileSystemState, input: &str) -> String {
    let completions = completion_engine(file_system_state, input);

    if completions.is_empty() {
        return input.to_string();
    }

    if completions.len() == 1 {
        return completions[0].clone();
    }

    // Find the longest common prefix among all completions
    let first = &completions[0];
    let mut common_prefix = String::new();

    for (i, ch) in first.chars().enumerate() {
        // Check if this character exists at the same position in all completions
        if completions.iter().all(|completion| {
            completion.chars().nth(i) == Some(ch)
        }) {
            common_prefix.push(ch);
        } else {
            break;
        }
    }

    common_prefix
}