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
        eprint!("DIR2>");
        std::io::stdin().read_line(&mut command).unwrap();
        let command: String = command.trim().to_string();
        if command.to_uppercase() == "EXIT" || command.to_uppercase() == "/E" {
            break;
        }
        if command.is_empty() {
            continue;
        }
        let tokens = parse_command(&command);
        println!("{:?}", tokens);
    }
}
