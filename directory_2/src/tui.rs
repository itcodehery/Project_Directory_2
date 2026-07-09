use ansi_to_tui::IntoText;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, EnableMouseCapture, DisableMouseCapture, MouseEventKind, MouseButton},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Flex, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Terminal,
};
use regex::Regex;
use std::io;

use crate::{
    commands::execute_command,
    completion,
    delegation::{execute_using_cmd, execute_with_piping},
    favorites::FavoritesManager,
    file_system_state::FileSystemState,
    parser::parse_command,
    utils,
};

pub fn run_tui(mut sys_state: FileSystemState, mut fav_manager: FavoritesManager) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut input = String::new();
    let mut cursor_index = 0;
    
    let mut completions: Vec<String> = Vec::new();
    let mut selected_completion = 0;
    let mut show_dropdown = false;

    let mut command_history: Vec<String> = Vec::new();
    let history_file = dirs::home_dir().map(|mut p| {
        p.push(".dir2_history");
        p
    });
    
    if let Some(ref path) = history_file {
        if let Ok(content) = std::fs::read_to_string(path) {
            command_history = content.lines().map(|s| s.to_string()).collect();
        }
    }
    
    let mut history_index: Option<usize> = None;
    let mut scroll_offset: usize = 0;

    // Terminal boilerplate
    crate::cprintln!("------------------------");
    crate::cprintln!("DIR2 Shell\nInstall the latest DIR2 for new features and improvements!");
    crate::cprintln!("------------------------");
    crate::cprintln!("Current State: {:?}", sys_state.get_current_state());

    loop {
        terminal.draw(|f| {
            let size = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),    // Logs
                    Constraint::Length(3), // Input box
                    Constraint::Length(1), // Bottom header
                ])
                .split(size);

            // Top Header
            let clear_marker = utils::get_clear_marker();
            
            let header_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Min(1)])
                .split(chunks[2]);

            let path_str = sys_state.get_current_path().to_string_lossy().to_string().replace("\"", "");
            let header_line = Line::from(vec![
                Span::styled("", Style::default().fg(Color::Green)),
                Span::styled(format!(" {} ", path_str), Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
                Span::styled("", Style::default().fg(Color::Green)),
            ]);
            let header = Paragraph::new(header_line)
                .alignment(Alignment::Center);
            f.render_widget(header, header_chunks[0]);

            let logs = utils::get_logs();
            let mut log_lines = Vec::new();
            
            let visible_logs = if clear_marker > 0 {
                &logs[clear_marker..]
            } else {
                &logs[..]
            };
            
            let height = chunks[0].height as usize;
            let end_index = visible_logs.len().saturating_sub(scroll_offset);
            let start_index = end_index.saturating_sub(height);
            
            for log in visible_logs.iter().skip(start_index).take(height) {
                if let Ok(text) = log.into_text() {
                    for line in text.lines {
                        log_lines.push(line);
                    }
                } else {
                    log_lines.push(Line::from(log.as_str()));
                }
            }
            
            let logs_paragraph = Paragraph::new(log_lines)
                .alignment(Alignment::Left);
            
            // If logs fits roughly around initial boilerplate height, it will be displayed at top,
            // but the user wants input in center if it's the start. Let's just always render logs
            // and have the input box at the bottom (like a standard TUI) or dynamically centered.
            // A standard TUI with input at bottom and logs above perfectly satisfies 
            // "center align all outputs" and is very clean.
            // Wait, "I want a text box at the center of the screen at the start."
            // If logs are just the 4 lines of boilerplate, we can use Flex layout to center it.
            let is_start = visible_logs.len() <= 5;
            
            let input_rect = if is_start {
                let center_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .flex(Flex::Center)
                    .constraints([Constraint::Length(3)])
                    .split(chunks[0]);
                f.render_widget(logs_paragraph, chunks[0]);
                center_chunks[0]
            } else {
                f.render_widget(logs_paragraph, chunks[0]);
                
                let max_scroll = visible_logs.len().saturating_sub(height);
                if max_scroll > 0 {
                    let mut scrollbar_state = ScrollbarState::default()
                        .content_length(max_scroll)
                        .position(max_scroll.saturating_sub(scroll_offset));
                    f.render_stateful_widget(
                        Scrollbar::default()
                            .orientation(ScrollbarOrientation::VerticalRight)
                            .begin_symbol(Some("▲"))
                            .end_symbol(Some("▼")),
                        chunks[0],
                        &mut scrollbar_state,
                    );
                }
                
                chunks[1]
            };

            let prompt = "> ".to_string();
            let display_text = format!("{}{}", prompt, input);
            
            let input_inner_width = input_rect.width.saturating_sub(2);
            let cursor_pos = prompt.len() + cursor_index;
            let input_scroll = if cursor_pos as u16 >= input_inner_width {
                (cursor_pos as u16).saturating_sub(input_inner_width) + 1
            } else {
                0
            };

            let input_widget = Paragraph::new(display_text)
                .scroll((0, input_scroll))
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
            
            // Render input
            f.render_widget(Clear, input_rect);
            f.render_widget(input_widget, input_rect);

            // Set cursor
            f.set_cursor_position(
                (input_rect.x + 1 + prompt.len() as u16 + cursor_index as u16 - input_scroll,
                 input_rect.y + 1)
            );

            // Dropdown
            if show_dropdown && !completions.is_empty() {
                let display_count = std::cmp::min(completions.len(), 10);
                let mut list_items = Vec::new();
                for (i, comp) in completions.iter().take(display_count).enumerate() {
                    let item = if i == selected_completion {
                        ListItem::new(format!("► {}", comp)).style(Style::default().bg(Color::White).fg(Color::Black))
                    } else {
                        ListItem::new(format!("  {}", comp))
                    };
                    list_items.push(item);
                }
                
                let list = List::new(list_items)
                    .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
                
                let mut drop_rect = input_rect;
                drop_rect.y = input_rect.y + 3;
                drop_rect.height = display_count as u16 + 2;
                if drop_rect.y + drop_rect.height > size.height {
                    drop_rect.y = input_rect.y.saturating_sub(drop_rect.height);
                }
                f.render_widget(Clear, drop_rect);
                f.render_widget(list, drop_rect);
            }

        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match (key.code, key.modifiers) {
                    (KeyCode::Enter, KeyModifiers::NONE) => {
                        let mut cmd = utils::substitute_env_vars(input.trim());
                        cmd = sys_state.expand_aliases(&cmd);
                        input.clear();
                        cursor_index = 0;
                        show_dropdown = false;
                        
                        if cmd.is_empty() {
                            continue;
                        }
                        
                        scroll_offset = 0;

                        if command_history.is_empty() || command_history.last().unwrap() != &cmd {
                            command_history.push(cmd.clone());
                            if let Some(ref path) = history_file {
                                if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
                                    use std::io::Write;
                                    let _ = writeln!(f, "{}", cmd);
                                }
                            }
                        }
                        history_index = None;

                        crate::cprintln!("");
                        crate::cprintln!("──────────────────────────────────────────────────");
                        crate::cprintln!("> {}", cmd);
                        if cmd.contains('|') {
                            if let Err(e) = execute_with_piping(&cmd) {
                                crate::cprintln!("Error: {}", e);
                            }
                            continue;
                        }

                        match parse_command(&cmd) {
                            Ok(parsed) => {
                                let res = execute_command(parsed, &mut sys_state, &mut fav_manager);
                                let _ = terminal.clear(); // Fix layout breaks after full-screen apps
                                if let Err(e) = res {
                                    crate::cprintln!("Error: {}", e);
                                } else if let Ok(s) = res {
                                    if s.to_uppercase() == "EXITED!" {
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                crate::cprintln!("Error: {}", e);
                            }
                        }
                    }
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        break;
                    }
                    (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                        input.insert(cursor_index, c);
                        cursor_index += 1;
                        
                        completions = completion::completion_engine(&mut sys_state, &input);
                        selected_completion = 0;
                        show_dropdown = !completions.is_empty();
                    }
                    (KeyCode::Backspace, KeyModifiers::NONE) => {
                        if cursor_index > 0 {
                            input.remove(cursor_index - 1);
                            cursor_index -= 1;
                            
                            completions = completion::completion_engine(&mut sys_state, &input);
                            selected_completion = 0;
                            show_dropdown = !completions.is_empty();
                        }
                    }
                    (KeyCode::Left, KeyModifiers::NONE) => {
                        if cursor_index > 0 {
                            cursor_index -= 1;
                        }
                    }
                    (KeyCode::Right, KeyModifiers::NONE) => {
                        if cursor_index < input.len() {
                            cursor_index += 1;
                        }
                    }
                    (KeyCode::Down, KeyModifiers::NONE) => {
                        if show_dropdown && !completions.is_empty() {
                            selected_completion = (selected_completion + 1) % completions.len();
                        } else if let Some(i) = history_index {
                            if i < command_history.len() - 1 {
                                let new_index = i + 1;
                                history_index = Some(new_index);
                                input = command_history[new_index].clone();
                                cursor_index = input.len();
                            } else {
                                history_index = None;
                                input.clear();
                                cursor_index = 0;
                            }
                        }
                    }
                    (KeyCode::Up, KeyModifiers::NONE) => {
                        if show_dropdown && !completions.is_empty() {
                            selected_completion = if selected_completion == 0 {
                                completions.len() - 1
                            } else {
                                selected_completion - 1
                            };
                        } else if !command_history.is_empty() {
                            let new_index = match history_index {
                                Some(i) => i.saturating_sub(1),
                                None => command_history.len() - 1,
                            };
                            history_index = Some(new_index);
                            input = command_history[new_index].clone();
                            cursor_index = input.len();
                        }
                    }
                    (KeyCode::Tab, KeyModifiers::NONE) => {
                        if show_dropdown && !completions.is_empty() {
                            if input.ends_with(' ') {
                                input.push_str(&completions[selected_completion]);
                            } else {
                                if let Some(last_space) = input.rfind(' ') {
                                    input.truncate(last_space + 1);
                                    input.push_str(&completions[selected_completion]);
                                } else {
                                    input = completions[selected_completion].clone();
                                }
                            }
                            completions.clear();
                            show_dropdown = false;
                            cursor_index = input.len();
                        } else {
                            if completion::auto_complete_single(&mut sys_state, &mut input) {
                                completions.clear();
                                show_dropdown = false;
                                cursor_index = input.len();
                            } else {
                                completions = completion::completion_engine(&mut sys_state, &input);
                                selected_completion = 0;
                                show_dropdown = !completions.is_empty();
                            }
                        }
                    }
                    (KeyCode::Esc, KeyModifiers::NONE) => {
                        show_dropdown = false;
                        completions.clear();
                    }
                    (KeyCode::PageUp, KeyModifiers::NONE) => {
                        let logs = utils::get_logs();
                        let clear_marker = utils::get_clear_marker();
                        let visible_len = if clear_marker > 0 { logs.len() - clear_marker } else { logs.len() };
                        let (_, rows) = crossterm::terminal::size().unwrap_or((80, 24));
                        let max_scroll = visible_len.saturating_sub(rows.saturating_sub(4) as usize);
                        scroll_offset = (scroll_offset + 5).min(max_scroll);
                    }
                    (KeyCode::PageDown, KeyModifiers::NONE) => {
                        scroll_offset = scroll_offset.saturating_sub(5);
                    }
                    _ => {}
                }
            }
        } else if let Event::Mouse(mouse_event) = event::read()? {
            match mouse_event.kind {
                MouseEventKind::ScrollUp => {
                    let logs = utils::get_logs();
                    let clear_marker = utils::get_clear_marker();
                    let visible_len = if clear_marker > 0 { logs.len() - clear_marker } else { logs.len() };
                    let (_, rows) = crossterm::terminal::size().unwrap_or((80, 24));
                    let max_scroll = visible_len.saturating_sub(rows.saturating_sub(4) as usize);
                    scroll_offset = (scroll_offset + 3).min(max_scroll);
                }
                MouseEventKind::ScrollDown => {
                    scroll_offset = scroll_offset.saturating_sub(3);
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
