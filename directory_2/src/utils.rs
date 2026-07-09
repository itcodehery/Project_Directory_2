use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref OUTPUT_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

pub static CLEAR_MARKER: AtomicUsize = AtomicUsize::new(0);

pub fn set_clear_marker() {
    if let Ok(logs) = OUTPUT_LOG.lock() {
        CLEAR_MARKER.store(logs.len(), Ordering::SeqCst);
    }
}

pub fn get_clear_marker() -> usize {
    CLEAR_MARKER.load(Ordering::SeqCst)
}

pub fn reset_clear_marker() {
    CLEAR_MARKER.store(0, Ordering::SeqCst);
}

pub fn push_log(text: &str) {
    if let Ok(mut logs) = OUTPUT_LOG.lock() {
        for line in text.split('\n') {
            logs.push(line.to_string());
        }
    }
}

pub fn get_logs() -> Vec<String> {
    if let Ok(logs) = OUTPUT_LOG.lock() {
        logs.clone()
    } else {
        vec![]
    }
}

pub fn clear_logs() {
    if let Ok(mut logs) = OUTPUT_LOG.lock() {
        logs.clear();
    }
}

pub fn substitute_env_vars(input: &str) -> String {
    let re = regex::Regex::new(r"\$([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    let mut replaced = re.replace_all(input, |caps: &regex::Captures| {
        std::env::var(&caps[1]).unwrap_or_default()
    }).to_string();
    
    let re_braced = regex::Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();
    replaced = re_braced.replace_all(&replaced, |caps: &regex::Captures| {
        std::env::var(&caps[1]).unwrap_or_default()
    }).to_string();
    
    replaced
}

#[macro_export]
macro_rules! cprintln {
    () => {
        crate::utils::push_log("");
    };
    ($($arg:tt)*) => {{
        let text = format!($($arg)*);
        crate::utils::push_log(&text);
    }};
}

#[macro_export]
macro_rules! cprint {
    ($($arg:tt)*) => {{
        let text = format!($($arg)*);
        crate::utils::push_log(&text);
    }};
}
