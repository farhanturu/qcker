use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use super::app::{ActiveTab, App, AppMode};

pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    match app.mode {
        AppMode::Normal => handle_normal_mode(app, key),
        AppMode::ConfirmDelete => handle_confirm_mode(app, key),
        AppMode::NewContainer => handle_new_container_mode(app, key),
        AppMode::ExecCommand => handle_exec_mode(app, key),
        AppMode::ImagePull => handle_pull_image_mode(app, key),
        AppMode::WatchingLogs => handle_logs_mode(app, key),
        _ => handle_normal_mode_fallback(app, key),
    }
}

pub fn handle_click_event(app: &mut App, mouse: MouseEvent) {
    let row = mouse.row;
    let col = mouse.column;

    match mouse.kind {
        MouseEventKind::Down(_) | MouseEventKind::Drag(_) => {
            if row == 0 {
                handle_button_click(app, col);
            } else if row >= 2 {
                let list_row = row - 2;
                match app.active_tab {
                    ActiveTab::Containers => {
                        if (list_row as usize) < app.containers.len() {
                            app.selected_index = list_row as usize;
                            app.selected_action = 0;
                        }
                    }
                    ActiveTab::Extensions => {
                        if (list_row as usize) < app.extensions.len() {
                            app.selected_index = list_row as usize;
                            app.selected_action = 0;
                        }
                    }
                    _ => {
                        if (list_row as usize) < app.max_items() {
                            app.selected_index = list_row as usize;
                        }
                    }
                }
            }
        }
        MouseEventKind::ScrollUp => { app.prev_item(); }
        MouseEventKind::ScrollDown => { app.next_item(); }
        _ => {}
    }
}

fn handle_button_click(app: &mut App, col: u16) {
    match app.active_tab {
        ActiveTab::Containers => {
            let actions = ["NEW","START","STOP","DEL","EXEC","LOGS"];
            let mut pos = 0u16;
            for (i, name) in actions.iter().enumerate() {
                let w = name.len() as u16 + 4;
                if col >= pos && col < pos + w {
                    app.selected_action = i;
                    execute_action(app, i);
                    return;
                }
                pos += w;
            }
        }
        ActiveTab::Extensions => {
            let actions = ["INST","ENBL","DSBL","UNST"];
            let mut pos = 0u16;
            for (i, name) in actions.iter().enumerate() {
                let w = name.len() as u16 + 4;
                if col >= pos && col < pos + w {
                    app.selected_action = i;
                    execute_action(app, i);
                    return;
                }
                pos += w;
            }
        }
        _ => {}
    }
}

fn execute_action(app: &mut App, action: usize) {
    match app.active_tab {
        ActiveTab::Containers => match action {
            0 => app.open_new_container(),
            1 => app.start_container(),
            2 => app.stop_container(),
            3 => app.delete_container(),
            4 => app.exec_in_container(),
            5 => app.watch_logs(),
            _ => {}
        },
        ActiveTab::Extensions => match action {
            0 => app.install_extension(),
            1 => app.enable_extension(),
            2 => app.disable_extension(),
            3 => app.uninstall_extension(),
            _ => {}
        },
        _ => {}
    }
}

fn handle_normal_mode_fallback(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => { app.should_quit = true; }
        KeyCode::Char('h') => { app.toggle_help(); }
        KeyCode::Char('r') => { app.refresh(); }
        KeyCode::Tab => { app.next_tab(); }
        KeyCode::BackTab => { app.prev_tab(); }
        KeyCode::Up | KeyCode::Char('k') => { app.prev_item(); }
        KeyCode::Down | KeyCode::Char('j') => { app.next_item(); }
        _ => {}
    }
}

fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            if app.show_help { app.toggle_help(); }
            else { app.should_quit = true; }
        }
        KeyCode::Char('h') => { app.toggle_help(); }
        KeyCode::Char('r') => { app.refresh(); }
        KeyCode::Tab => { app.next_tab(); }
        KeyCode::BackTab => { app.prev_tab(); }
        KeyCode::Up | KeyCode::Char('k') => { app.prev_item(); }
        KeyCode::Down | KeyCode::Char('j') => { app.next_item(); }
        KeyCode::Enter => {
            match app.active_tab {
                ActiveTab::Containers => {
                    if app.get_selected_container().is_some() {
                        let action = app.selected_action;
                        execute_action(app, action);
                    }
                }
                ActiveTab::Extensions => {
                    let action = app.selected_action;
                    execute_action(app, action);
                }
                _ => {
                    let mi = app.max_items();
                    if mi > 0 { app.selected_index = (app.selected_index + 1).min(mi - 1); }
                }
            }
        }
        KeyCode::Char('n') => { if app.active_tab == ActiveTab::Containers { app.open_new_container(); } }
        KeyCode::Char('p') => {
            if app.active_tab == ActiveTab::Images {
                app.mode = AppMode::ImagePull;
                app.pull_input.clear();
                app.status_message = "Image:".to_string();
            }
        }
        KeyCode::Char('s') => { if app.active_tab == ActiveTab::Containers { app.start_container(); } }
        KeyCode::Char('x') => { if app.active_tab == ActiveTab::Containers { app.stop_container(); } }
        KeyCode::Char('d') => {
            match app.active_tab {
                ActiveTab::Containers => { app.delete_container(); }
                ActiveTab::Extensions => { app.disable_extension(); }
                _ => {}
            }
        }
        KeyCode::Char('t') => { if app.active_tab == ActiveTab::Containers { /* terminal not available yet */ app.status_message = "Terminal: use 'qcker exec'".to_string(); } }
        KeyCode::Char('f') => { if app.active_tab == ActiveTab::Containers { app.status_message = "Files: use 'qcker exec <id> ls'".to_string(); } }
        KeyCode::Char('i') => {
            match app.active_tab {
                ActiveTab::Containers => { app.exec_in_container(); }
                ActiveTab::Extensions => { app.install_extension(); }
                _ => {}
            }
        }
        KeyCode::Char('w') => { if app.active_tab == ActiveTab::Containers { app.watch_logs(); } }
        KeyCode::Char('e') => { if app.active_tab == ActiveTab::Extensions { app.enable_extension(); } }
        KeyCode::Char('u') => { if app.active_tab == ActiveTab::Extensions { app.uninstall_extension(); } }
        KeyCode::Left => {
            if app.selected_action > 0 { app.selected_action -= 1; }
        }
        KeyCode::Right => {
            let max = match app.active_tab {
                ActiveTab::Containers => 5,
                ActiveTab::Extensions => 3,
                _ => 0,
            };
            if app.selected_action < max { app.selected_action += 1; }
        }
        _ => {}
    }
}

fn handle_confirm_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => { app.execute_confirm(); }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => { app.cancel_confirm(); }
        _ => {}
    }
}

fn handle_new_container_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => { app.exit_new_container(); }
        KeyCode::Enter => { app.execute_new_container(); }
        KeyCode::Tab => {
            if app.new_name.is_empty() {
                app.new_name = "my-container".to_string();
                app.status_message = "Name set. Tab for Image".to_string();
            } else if app.new_image.is_empty() {
                app.new_image = "alpine:latest".to_string();
                app.status_message = "Image set. Tab for Command".to_string();
            } else {
                app.status_message = "Ready to create!".to_string();
            }
        }
        KeyCode::Char(c) => {
            if app.new_cmd.is_empty() {
                if app.new_image.is_empty() { app.new_image.push(c); }
                else { app.new_name.push(c); }
            } else { app.new_cmd.push(c); }
        }
        KeyCode::Backspace => {
            if app.new_cmd.len() > 0 { app.new_cmd.pop(); }
            else if app.new_image.len() > 0 { app.new_image.pop(); }
            else if app.new_name.len() > 0 { app.new_name.pop(); }
        }
        _ => {}
    }
}

fn handle_exec_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => { app.exec_cmd.clear(); app.mode = AppMode::Normal; app.status_message = "Cancelled".to_string(); }
        KeyCode::Enter => { app.execute_exec(); }
        KeyCode::Char(c) => { app.exec_cmd.push(c); app.status_message = format!("> {}", app.exec_cmd); }
        KeyCode::Backspace => { app.exec_cmd.pop(); app.status_message = format!("> {}", app.exec_cmd); }
        _ => {}
    }
}

fn handle_pull_image_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => { app.pull_input.clear(); app.mode = AppMode::Normal; app.status_message = "Cancelled".to_string(); }
        KeyCode::Enter => { app.pull_image(); }
        KeyCode::Char(c) => { app.pull_input.push(c); app.status_message = format!("Pulling: {}", app.pull_input); }
        KeyCode::Backspace => { app.pull_input.pop(); app.status_message = format!("Pulling: {}", app.pull_input); }
        _ => {}
    }
}

fn handle_logs_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => { app.mode = AppMode::Normal; app.status_message = "Stopped watching".to_string(); }
        KeyCode::Up | KeyCode::Char('k') => { app.scroll_offset = app.scroll_offset.saturating_sub(1); }
        KeyCode::Down | KeyCode::Char('j') => { app.scroll_offset += 1; }
        KeyCode::Char('g') => { app.scroll_offset = 0; }
        KeyCode::Char('G') => { app.scroll_offset = app.logs.len().saturating_sub(20); }
        KeyCode::Char('r') => {
            if let Some(id) = &app.selected_container {
                let _ = app.run_command(&["logs", id]);
                app.refresh();
            }
        }
        _ => {}
    }
}
