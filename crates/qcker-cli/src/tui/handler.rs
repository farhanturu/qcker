use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};

use super::app::{App, AppMode, ActiveTab};

pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    match app.mode {
        AppMode::Normal => handle_normal_mode(app, key),
        AppMode::ContainerFiles => handle_file_browser_mode(app, key),
        AppMode::FileEditor => handle_editor_mode(app, key),
        AppMode::CommandInput => handle_command_mode(app, key),
        AppMode::ConfirmAction => handle_confirm_mode(app, key),
    }
}

pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent) {
    match app.mode {
        AppMode::Normal | AppMode::ContainerFiles => {
            handle_normal_mouse(app, mouse);
        }
        AppMode::FileEditor => {}
        AppMode::CommandInput => {}
        AppMode::ConfirmAction => {}
    }
}

fn handle_normal_mouse(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let row = mouse.row as usize;
            let col = mouse.column as usize;

            if row == 0 || row == 1 {
                let tab_index = col / 13;
                app.click_tab(tab_index);
            } else if row >= 3 {
                let item_row = row - 3 + app.scroll_offset;
                app.click_item(item_row);
            }
        }
        MouseEventKind::ScrollUp => {
            app.prev_item();
        }
        MouseEventKind::ScrollDown => {
            app.next_item();
        }
        _ => {}
    }
}

fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('h') => {
            app.show_help = !app.show_help;
        }
        KeyCode::Char('r') => {
            app.refresh();
        }
        KeyCode::Char('a') => {
            app.toggle_auto_refresh();
        }
        KeyCode::Tab => {
            app.next_tab();
        }
        KeyCode::BackTab => {
            app.prev_tab();
        }
        KeyCode::Up => {
            app.prev_item();
        }
        KeyCode::Down => {
            app.next_item();
        }
        KeyCode::Char('j') => {
            app.next_item();
        }
        KeyCode::PageDown => {
            app.page_down();
        }
        KeyCode::PageUp => {
            app.page_up();
        }
        KeyCode::Enter => {
            match app.active_tab {
                ActiveTab::Containers => app.open_container_files(),
                ActiveTab::Marketplace => app.confirm_uninstall_extension(),
                _ => {}
            }
        }
        KeyCode::Char('s') => {
            if app.active_tab == ActiveTab::Containers {
                app.stop_container();
            }
        }
        KeyCode::Char('x') => {
            if app.active_tab == ActiveTab::Containers {
                app.kill_container();
            }
        }
        KeyCode::Delete | KeyCode::Char('d') => {
            if app.active_tab == ActiveTab::Containers {
                app.delete_container();
            }
        }
        KeyCode::Char('u') => {
            if app.active_tab == ActiveTab::Marketplace {
                app.confirm_uninstall_extension();
            }
        }
        KeyCode::Char('g') => {
            if app.active_tab == ActiveTab::Logs {
                app.scroll_offset = 0;
            }
        }
        KeyCode::Char('G')
            if app.active_tab == ActiveTab::Logs => {
                app.scroll_offset = app.logs.len().saturating_sub(20);
            }
        _ => {}
    }
}

fn handle_file_browser_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.exit_container_files();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.prev_item();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_item();
        }
        KeyCode::PageDown => {
            app.page_down();
        }
        KeyCode::PageUp => {
            app.page_up();
        }
        KeyCode::Enter => {
            if app.selected_index == 0 && app.current_path != "/" {
                app.navigate_up();
            } else {
                app.navigate_into();
            }
        }
        KeyCode::Backspace => {
            app.navigate_up();
        }
        KeyCode::Char('e') => {
            app.open_file_editor();
        }
        KeyCode::Char('d') => {
            app.confirm_delete();
        }
        KeyCode::Char('n') => {
            app.create_new_file();
        }
        KeyCode::Char('m') => {
            app.create_new_dir();
        }
        KeyCode::Char('r') => {
            app.refresh();
        }
        KeyCode::Char('h') => {
            app.show_help = !app.show_help;
        }
        _ => {}
    }
}

fn handle_editor_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.close_editor();
        }
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.save_editor();
        }
        KeyCode::Char(c) => {
            app.editor_content.push(c);
            app.editor_cursor_x += 1;
            app.editor_modified = true;
        }
        KeyCode::Enter => {
            app.editor_content.push('\n');
            app.editor_cursor_x = 0;
            app.editor_cursor_y += 1;
            app.editor_modified = true;
        }
        KeyCode::Backspace => {
            if app.editor_cursor_x > 0 {
                app.editor_content.pop();
                app.editor_cursor_x -= 1;
                app.editor_modified = true;
            }
        }
        KeyCode::Tab => {
            app.editor_content.push_str("    ");
            app.editor_cursor_x += 4;
            app.editor_modified = true;
        }
        KeyCode::Left => {
            if app.editor_cursor_x > 0 {
                app.editor_cursor_x -= 1;
            }
        }
        KeyCode::Right => {
            app.editor_cursor_x += 1;
        }
        KeyCode::Up => {
            if app.editor_cursor_y > 0 {
                app.editor_cursor_y -= 1;
            }
        }
        KeyCode::Down => {
            app.editor_cursor_y += 1;
        }
        _ => {}
    }
}

fn handle_command_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.command_input.clear();
            app.mode = AppMode::ContainerFiles;
        }
        KeyCode::Enter => {
            app.execute_command();
        }
        KeyCode::Char(c) => {
            app.command_input.push(c);
        }
        KeyCode::Backspace => {
            app.command_input.pop();
        }
        _ => {}
    }
}

fn handle_confirm_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.execute_confirm();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.cancel_confirm();
        }
        _ => {}
    }
}
