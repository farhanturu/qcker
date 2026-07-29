use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Tabs},
    Frame,
};

use super::app::{ActiveTab, App, AppMode};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);

    match app.mode {
        AppMode::ConfirmDelete => draw_confirm_popup(f, app),
        AppMode::CommandInput => draw_command_popup(f, app),
        _ => {}
    }

    if app.show_help {
        draw_help_popup(f);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec![
        "Containers",
        "Images",
        "Networks",
        "Volumes",
        "Files",
        "Editor",
        "Marketplace",
        "Logs",
    ];

    let selected = match app.active_tab {
        ActiveTab::Containers => 0,
        ActiveTab::Images => 1,
        ActiveTab::Networks => 2,
        ActiveTab::Volumes => 3,
        ActiveTab::Files => 4,
        ActiveTab::Editor => 5,
        ActiveTab::Marketplace => 6,
        ActiveTab::Logs => 7,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Qcker"))
        .select(selected)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    f.render_widget(tabs, area);
}

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    match app.active_tab {
        ActiveTab::Containers => draw_containers(f, app, area),
        ActiveTab::Images => draw_images(f, app, area),
        ActiveTab::Networks => draw_networks(f, app, area),
        ActiveTab::Volumes => draw_volumes(f, app, area),
        ActiveTab::Files => draw_files(f, app, area),
        ActiveTab::Editor => draw_editor(f, app, area),
        ActiveTab::Marketplace => draw_marketplace(f, app, area),
        ActiveTab::Logs => draw_logs(f, app, area),
    }
}

fn draw_containers(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("ID"),
        Cell::from("NAME"),
        Cell::from("STATUS"),
        Cell::from("IMAGE"),
        Cell::from("PID"),
        Cell::from("CREATED"),
    ])
    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    .height(1);

    let rows: Vec<Row> = app.containers.iter().enumerate().map(|(i, c)| {
        let style = if i == app.selected_index {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default().fg(Color::White)
        };

        let status_style = match c.status.as_str() {
            "running" => Style::default().fg(Color::Green),
            "created" => Style::default().fg(Color::Yellow),
            "stopped" => Style::default().fg(Color::Red),
            _ => Style::default().fg(Color::Gray),
        };

        Row::new(vec![
            Cell::from(truncate(&c.id, 12)),
            Cell::from(truncate(&c.name, 18)),
            Cell::from(Span::styled(truncate(&c.status, 10), status_style)),
            Cell::from(truncate(&c.image, 18)),
            Cell::from(c.pid.map_or("-".to_string(), |p| p.to_string())),
            Cell::from(truncate(&c.created, 16)),
        ])
        .style(style)
        .height(1)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(13),
            Constraint::Length(19),
            Constraint::Length(11),
            Constraint::Length(19),
            Constraint::Length(8),
            Constraint::Min(17),
        ],
    )
    .header(header)
    .block(Block::default()
        .borders(Borders::ALL)
        .title(format!("Containers ({})", app.containers.len())));

    f.render_widget(table, area);
}

fn draw_images(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("ID"),
        Cell::from("TAGS"),
        Cell::from("SIZE"),
        Cell::from("CREATED"),
    ])
    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    .height(1);

    let rows: Vec<Row> = app.images.iter().enumerate().map(|(i, img)| {
        let style = if i == app.selected_index {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default().fg(Color::White)
        };

        Row::new(vec![
            Cell::from(truncate(&img.id, 12)),
            Cell::from(truncate(&img.tags, 30)),
            Cell::from(truncate(&img.size, 10)),
            Cell::from(truncate(&img.created, 16)),
        ])
        .style(style)
        .height(1)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(13),
            Constraint::Min(31),
            Constraint::Length(11),
            Constraint::Length(17),
        ],
    )
    .header(header)
    .block(Block::default()
        .borders(Borders::ALL)
        .title(format!("Images ({})", app.images.len())));

    f.render_widget(table, area);
}

fn draw_networks(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("ID"),
        Cell::from("NAME"),
        Cell::from("DRIVER"),
        Cell::from("SUBNET"),
    ])
    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    .height(1);

    let rows: Vec<Row> = app.networks.iter().enumerate().map(|(i, n)| {
        let style = if i == app.selected_index {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default().fg(Color::White)
        };

        Row::new(vec![
            Cell::from(truncate(&n.id, 12)),
            Cell::from(truncate(&n.name, 18)),
            Cell::from(truncate(&n.driver, 10)),
            Cell::from(truncate(&n.subnet, 18)),
        ])
        .style(style)
        .height(1)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(13),
            Constraint::Length(19),
            Constraint::Length(11),
            Constraint::Min(19),
        ],
    )
    .header(header)
    .block(Block::default()
        .borders(Borders::ALL)
        .title(format!("Networks ({})", app.networks.len())));

    f.render_widget(table, area);
}

fn draw_volumes(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("NAME"),
        Cell::from("DRIVER"),
        Cell::from("MOUNTPOINT"),
    ])
    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    .height(1);

    let rows: Vec<Row> = app.volumes.iter().enumerate().map(|(i, v)| {
        let style = if i == app.selected_index {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default().fg(Color::White)
        };

        Row::new(vec![
            Cell::from(truncate(&v.name, 18)),
            Cell::from(truncate(&v.driver, 10)),
            Cell::from(truncate(&v.mountpoint, 40)),
        ])
        .style(style)
        .height(1)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(19),
            Constraint::Length(11),
            Constraint::Min(41),
        ],
    )
    .header(header)
    .block(Block::default()
        .borders(Borders::ALL)
        .title(format!("Volumes ({})", app.volumes.len())));

    f.render_widget(table, area);
}

fn draw_files(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    let mut items: Vec<ListItem> = Vec::new();

    if app.current_path != "/" {
        items.push(ListItem::new(Line::from(Span::styled(
            ".. (parent)",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ))));
    }

    for (i, file) in app.files.iter().enumerate() {
        let style = if i == app.selected_index {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else if file.is_dir {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };

        let icon = if file.is_dir { "d" } else { "f" };
        let size = if file.is_dir {
            String::new()
        } else {
            format_size(file.size)
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("[{}] ", icon), style),
            Span::styled(truncate(&file.name, 20), style),
            Span::styled(format!("  {}", size), Style::default().fg(Color::Gray)),
        ])));
    }

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(truncate(&format!("Files: {}", app.current_path), 40)));

    f.render_widget(list, chunks[0]);

    let preview = if let Some(file) = app.files.get(app.selected_index) {
        if file.is_dir {
            "<Directory>".to_string()
        } else {
            let rootfs = app.data_dir.join("containers")
                .join(app.selected_container.as_deref().unwrap_or(""))
                .join("rootfs");
            let full_path = rootfs.join(file.path.trim_start_matches('/'));

            if let Ok(content) = std::fs::read_to_string(&full_path) {
                let truncated = if content.len() > 2000 {
                    format!("{}...\n\n[Truncated - {} bytes total]", &content[..2000], content.len())
                } else {
                    content
                };
                truncated
            } else {
                "<Cannot read file>".to_string()
            }
        }
    } else {
        "<Select a file>".to_string()
    };

    let preview_widget = Paragraph::new(preview)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Preview"))
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(preview_widget, chunks[1]);
}

fn draw_editor(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let lines: Vec<Line> = app.editor_content
        .lines()
        .enumerate()
        .map(|(i, line)| {
            let line_num = format!("{:4} ", i + 1);
            Line::from(vec![
                Span::styled(line_num, Style::default().fg(Color::DarkGray)),
                Span::styled(truncate(line, 100), Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let editor = Paragraph::new(lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(if app.editor_modified {
                "Editor *"
            } else {
                "Editor"
            }))
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: false });

    f.render_widget(editor, chunks[0]);

    let status = format!(
        "Ln {} Col {} | Ctrl+S:Save Esc:Close",
        app.editor_cursor_y + 1,
        app.editor_cursor_x + 1
    );

    let status_bar = Paragraph::new(status)
        .style(Style::default().fg(Color::DarkGray).bg(Color::Black));

    f.render_widget(status_bar, chunks[1]);
}

fn draw_marketplace(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("NAME"),
        Cell::from("VERSION"),
        Cell::from("CATEGORY"),
        Cell::from("STATUS"),
        Cell::from("DESCRIPTION"),
    ])
    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    .height(1);

    let rows: Vec<Row> = app.marketplace.iter().enumerate().map(|(i, ext)| {
        let style = if i == app.selected_index {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default().fg(Color::White)
        };

        let status = if ext.built_in {
            Span::styled("Built-in", Style::default().fg(Color::Green))
        } else if ext.installed {
            Span::styled("Installed", Style::default().fg(Color::Cyan))
        } else {
            Span::styled("Available", Style::default().fg(Color::Yellow))
        };

        Row::new(vec![
            Cell::from(truncate(&ext.name, 16)),
            Cell::from(truncate(&ext.version, 8)),
            Cell::from(truncate(&ext.category, 10)),
            Cell::from(status),
            Cell::from(truncate(&ext.description, 30)),
        ])
        .style(style)
        .height(1)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(17),
            Constraint::Length(9),
            Constraint::Length(11),
            Constraint::Length(11),
            Constraint::Min(31),
        ],
    )
    .header(header)
    .block(Block::default()
        .borders(Borders::ALL)
        .title(format!("Extensions ({})", app.marketplace.len())));

    f.render_widget(table, area);
}

fn draw_logs(f: &mut Frame, app: &App, area: Rect) {
    let logs: Vec<Line> = app.logs.iter().map(|l| {
        Line::from(Span::styled(truncate(l, 120), Style::default().fg(Color::White)))
    }).collect();

    let paragraph = Paragraph::new(logs)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Logs"))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.mode {
        AppMode::Normal => {
            match app.active_tab {
                super::app::ActiveTab::Marketplace => "q:Quit Tab:Switch u:Uninstall r:Refresh h:Help",
                super::app::ActiveTab::Containers => "q:Quit Tab:Switch Enter:Files r:Refresh h:Help",
                _ => "q:Quit Tab:Switch r:Refresh h:Help",
            }
        },
        AppMode::ContainerFiles => "q:Back Enter:Open d:Delete n:New m:Mkdir Esc:Exit",
        AppMode::FileEditor => "Ctrl+S:Save Esc:Close",
        AppMode::CommandInput => "Enter:Confirm Esc:Cancel",
        AppMode::ConfirmDelete => "y:Yes n:No",
    };

    let status = truncate(&app.status_message, 50);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(status, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled(help_text, Style::default().fg(Color::Gray)),
    ]))
    .block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}

fn draw_confirm_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(40, 20, f.area());

    let text = vec![
        Line::from(Span::styled(
            truncate(&app.confirm_message, 30),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "y = Yes, n = No",
            Style::default().fg(Color::Gray),
        )),
    ];

    let popup = Paragraph::new(text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Confirm")
            .style(Style::default().bg(Color::DarkGray)))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(Clear, area);
    f.render_widget(popup, area);
}

fn draw_command_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 15, f.area());

    let text = vec![
        Line::from(Span::styled(
            truncate(&app.status_message, 40),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Cyan)),
            Span::styled(truncate(&app.command_input, 30), Style::default().fg(Color::White)),
            Span::styled("_", Style::default().fg(Color::White).add_modifier(Modifier::SLOW_BLINK)),
        ]),
    ];

    let popup = Paragraph::new(text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Input")
            .style(Style::default().bg(Color::DarkGray)));

    f.render_widget(Clear, area);
    f.render_widget(popup, area);
}

fn draw_help_popup(f: &mut Frame) {
    let area = centered_rect(50, 50, f.area());

    let help_text = vec![
        Line::from(Span::styled("Qcker TUI Help", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation:", Style::default().fg(Color::Yellow))),
        Line::from("  Tab        Switch tabs"),
        Line::from("  Up/Down    Navigate"),
        Line::from(""),
        Line::from(Span::styled("Containers:", Style::default().fg(Color::Yellow))),
        Line::from("  Enter      Browse files"),
        Line::from(""),
        Line::from(Span::styled("Files:", Style::default().fg(Color::Yellow))),
        Line::from("  Enter      Open"),
        Line::from("  e          Edit"),
        Line::from("  d          Delete"),
        Line::from("  n          New file"),
        Line::from("  m          New dir"),
        Line::from(""),
        Line::from(Span::styled("Extensions:", Style::default().fg(Color::Yellow))),
        Line::from("  u/Enter    Uninstall"),
        Line::from(""),
        Line::from(Span::styled("General:", Style::default().fg(Color::Yellow))),
        Line::from("  r          Refresh"),
        Line::from("  h          Help"),
        Line::from("  q          Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Help")
            .style(Style::default().bg(Color::DarkGray)));

    f.render_widget(Clear, area);
    f.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
