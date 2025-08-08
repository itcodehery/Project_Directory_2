use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use file_system_state::FileSystemState;
use std::io::{self, stdout, Write};
use crossterm::event::KeyModifiers;
use crate::{completion, file_system_state};

pub fn read_line(input: &String, file_system_state: &mut FileSystemState) -> io::Result<String> {
    // Enable raw mode for precise terminal control
    terminal::enable_raw_mode()?;

    // Clone the input string to create a mutable copy
    let mut line = input.clone();
    let mut completions: Vec<String> = Vec::new();
    let mut selected_completion = 0;
    let mut show_dropdown = false;
    let mut cursor_index = line.len();

    // Get initial cursor position
    let (initial_col, initial_row) = cursor::position()?;

    let result = loop {
        // Read a keyboard event and match only on key press events

        if let Event::Key(KeyEvent {
                              code,
                              kind: KeyEventKind::Press,
            modifiers,
                              ..
                          }) = event::read()?
        {
            match (code,modifiers) {
                (KeyCode::Enter, KeyModifiers::NONE) => {
                    // Clear dropdown if showing
                    if show_dropdown {
                        clear_dropdown(&completions, initial_row)?;
                    }
                    break Ok(line);
                }

                (KeyCode::Backspace, KeyModifiers::NONE) => {
                    if cursor_index > 0 {
                        line.remove(cursor_index - 1);
                        cursor_index -= 1;

                        completions = completion::completion_engine(file_system_state, &line);
                        selected_completion = 0;
                        show_dropdown = !completions.is_empty();
                        redraw_input_and_dropdown(&line, &completions, selected_completion, initial_col, initial_row, show_dropdown, &cursor_index)?;

                        execute!(stdout(), cursor::MoveTo(initial_col + cursor_index as u16, initial_row))?;
                    }
                }


                (KeyCode::Char(c), ..) => {
                    line.insert(cursor_index, c);
                    cursor_index += 1;

                    completions = completion::completion_engine(file_system_state, &line);
                    selected_completion = 0;
                    show_dropdown = !completions.is_empty();
                    redraw_input_and_dropdown(&line, &completions, selected_completion, initial_col, initial_row, show_dropdown,&cursor_index)?;

                    execute!(stdout(), cursor::MoveTo(initial_col + cursor_index as u16, initial_row))?;
                }

                (KeyCode::Tab, KeyModifiers::NONE) => {
                    if show_dropdown && !completions.is_empty() {
                        // Replace only the last word with the selected completion
                        // let words: Vec<&str> = line.split_whitespace().collect();
                        if line.ends_with(' ') {
                            // Add the completion as a new word
                            line.push_str(&completions[selected_completion]);
                        } else {
                            // Replace the last word
                            if let Some(last_space_pos) = line.rfind(' ') {
                                line.truncate(last_space_pos + 1);
                                line.push_str(&completions[selected_completion]);
                            } else {
                                // No spaces, replace entire line
                                line = completions[selected_completion].clone();
                            }
                        }
                        completions.clear();
                        show_dropdown = false;
                        cursor_index = line.len();
                        redraw_input_and_dropdown(&line, &completions, selected_completion, initial_col, initial_row, show_dropdown,&line.len())?;
                    }
                    else {
                        // Try single auto-completion if no dropdown is showing
                        if completion::auto_complete_single(file_system_state, &mut line) {
                            // Clear any existing dropdown and redraw with completed input
                            completions.clear();
                            show_dropdown = false;
                            redraw_input_and_dropdown(&line, &completions, selected_completion, initial_col, initial_row, show_dropdown,&cursor_index)?;
                        } else {
                            // Show dropdown with available completions
                            completions = completion::completion_engine(file_system_state, &line);
                            selected_completion = 0;
                            show_dropdown = !completions.is_empty();
                            redraw_input_and_dropdown(&line, &completions, selected_completion, initial_col, initial_row, show_dropdown,&cursor_index)?;
                        }
                    }
                }

                (KeyCode::Down, KeyModifiers::NONE) => {
                    if show_dropdown && !completions.is_empty() {
                        selected_completion = (selected_completion + 1) % completions.len();
                        redraw_dropdown(&completions, selected_completion, initial_row)?;
                    }
                }

                (KeyCode::Up, KeyModifiers::NONE) => {
                    if show_dropdown && !completions.is_empty() {
                        selected_completion = if selected_completion == 0 {
                            completions.len() - 1
                        } else {
                            selected_completion - 1
                        };
                        redraw_dropdown(&completions, selected_completion, initial_row)?;
                    }
                }

                (KeyCode::Left, KeyModifiers::NONE) => {
                    if cursor_index > 0 {
                        cursor_index -= 1;
                        execute!(stdout(), cursor::MoveTo(initial_col + cursor_index as u16, initial_row))?;
                    }
                }

                (KeyCode::Right, KeyModifiers::NONE) => {
                    if cursor_index < line.len() {
                        cursor_index += 1;
                        execute!(stdout(), cursor::MoveTo(initial_col + cursor_index as u16, initial_row))?;
                    }

                }



                (KeyCode::Esc, KeyModifiers::NONE) => {
                    if show_dropdown {
                        clear_dropdown(&completions, initial_row)?;
                        show_dropdown = false;
                        completions.clear();
                    }
                }

                (KeyCode::Backspace, KeyModifiers::CONTROL) => {
                    if cursor_index > 0 {
                        let mut new_index = cursor_index;
                        while new_index > 0 && line.chars().nth(new_index - 1).unwrap().is_whitespace() {
                            new_index -= 1;
                        }
                        while new_index > 0 && !line.chars().nth(new_index - 1).unwrap().is_whitespace() {
                            new_index -= 1;
                        }

                        line.replace_range(new_index..cursor_index, "");
                        cursor_index = new_index;

                        redraw_input_and_dropdown(
                            &line,
                            &completions,
                            selected_completion,
                            initial_col,
                            initial_row,
                            show_dropdown,
                            &cursor_index,
                        )?;
                    }
                }

                _ => {}
            }
        }


    };

    // Cleanup: disable raw mode
    terminal::disable_raw_mode()?;
    result
}

fn redraw_input_and_dropdown(
    line: &str,
    completions: &[String],
    selected: usize,
    initial_col: u16,
    initial_row: u16,
    show_dropdown: bool,
    cursor_index: &usize
) -> io::Result<()> {
    // Clear current line and any existing dropdown
    execute!(stdout(), cursor::MoveTo(initial_col, initial_row))?;
    execute!(stdout(), Clear(ClearType::FromCursorDown))?;

    // Redraw input line
    print!("{}", line);
    stdout().flush()?;

    // Draw dropdown if needed
    if show_dropdown && !completions.is_empty() {
        draw_dropdown(completions, selected, initial_row)?;
    }

    // Move the cursor back to the end of input
    // execute!(stdout(), cursor::MoveTo(initial_col + line.len() as u16, initial_row))?;
    execute!(stdout(), cursor::MoveTo(initial_col + *cursor_index as u16, initial_row))?;
    Ok(())
}

fn draw_dropdown(completions: &[String], selected: usize, input_row: u16) -> io::Result<()> {
    let max_display = 10; // Maximum number of completions to show
    let display_count = std::cmp::min(completions.len(), max_display);

    for (i, completion) in completions.iter().take(display_count).enumerate() {
        execute!(stdout(), cursor::MoveTo(0, input_row + 1 + i as u16))?;

        if i == selected {
            // Highlight selected item
            execute!(stdout(), SetForegroundColor(Color::Black))?;
            execute!(stdout(), crossterm::style::SetBackgroundColor(Color::White))?;
            print!("► {}", completion);
            execute!(stdout(), ResetColor)?;
        } else {
            print!("  {}", completion);
        }
    }

    stdout().flush()?;
    Ok(())
}

fn redraw_dropdown(completions: &[String], selected: usize, input_row: u16) -> io::Result<()> {
    draw_dropdown(completions, selected, input_row)?;
    Ok(())
}

fn clear_dropdown(completions: &[String], input_row: u16) -> io::Result<()> {
    let max_display = 10;
    let display_count = std::cmp::min(completions.len(), max_display);

    for i in 0..display_count {
        execute!(stdout(), cursor::MoveTo(0, input_row + 1 + i as u16))?;
        execute!(stdout(), Clear(ClearType::CurrentLine))?;
    }

    Ok(())
}