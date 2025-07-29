use rust_search::SearchBuilder;
use crate::file_system_state::FileSystemState;

#[derive(Debug)]
pub enum SearchEngine{
    Google,
    DuckDuckGo,
    ChatGPT,
    Perplexity,
}

impl SearchEngine {
    pub fn to_string(&self) -> String{
        match self {
            &SearchEngine::Google => String::from("Google"),
            &SearchEngine::DuckDuckGo => String::from("DuckDuckGo"),
            &SearchEngine::ChatGPT => String::from("ChatGPT"),
            &SearchEngine::Perplexity => String::from("Perplexity"),
        }
    }
}
pub fn search_builder(sys_state: &mut FileSystemState, query_string : &str, ) -> Vec<String>  {
    let search: Vec<String> = SearchBuilder::default()
        .location("/").search_input(query_string).limit(10).hidden().build().collect();

    return search;
}
