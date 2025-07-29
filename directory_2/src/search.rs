use rust_search::SearchBuilder;
use crate::file_system_state::FileSystemState;

pub fn search_builder(sys_state: &mut FileSystemState, query_string : &str, ) -> Vec<String>  {
    let search: Vec<String> = SearchBuilder::default()
        .location("/").search_input(query_string).limit(10).hidden().build().collect();

    return search;
}