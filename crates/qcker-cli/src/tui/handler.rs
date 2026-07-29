use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::{App, AppMode};

pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    match app.mode {
        AppMode::Normal => handle_normal_mode(app, key),
        AppMode::ContainerFiles => handle_file_browser_mode(app, key),
        AppMode::FileEditor => handle_editor_mode(app, key),
        AppMode::CommandInput => handle_command_mode(app, key),
        AppMode::ConfirmDelete => handle_confirm_mode(app, key),
    }
}

fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        KeyCode::Char('h') => {
            app.show_help = !app.show_help;
        }
        KeyCode::Char('r') => {
            app.refresh();
        }
        KeyCode::Tab => {
            app.next_tab();
        }
        KeyCode::BackTab => {
            app.prev_tab();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.prev_item();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_item();
        }
        KeyCode::Enter => {
            match app.active_tab {
                super::app::ActiveTab::Containers => app.open_container_files(),
                super::app::ActiveTab::Marketplace => app.confirm_uninstall_extension(),
                _ => {}
            }
        }
        KeyCode::Char('u') => {
            if let super::app::ActiveTab::Marketplace = app.active_tab {
                app.confirm_uninstall_extension();
            }
        }
        _ => {}
    }
}

fn handle_file_browser_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.exit_container_files();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.prev_item();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_item();
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
