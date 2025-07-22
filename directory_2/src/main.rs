use std::path::PathBuf;

mod parser;

use parser::parse_command;
struct State {
    state: Option<PathBuf>,
}
fn main() {
    let mut current_state: State = State { state: None };
    command_handler(&mut current_state);
}

fn command_handler(state: &mut State) {
    loop {
        let mut command: String = String::new();
        std::io::stdin().read_line(&mut command).unwrap();
        let command: String = command.trim().to_string();
        if command == "dir2 exit" {
            break;
        }
    }
}
