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
            .title("Qcker Container Engine")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
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
            Cell::from(c.id[..12.min(c.id.len())].to_string()),
            Cell::from(c.name.clone()),
            Cell::from(Span::styled(c.status.clone(), status_style)),
            Cell::from(c.image.clone()),
            Cell::from(c.pid.map_or("-".to_string(), |p| p.to_string())),
            Cell::from(c.created[..19.min(c.created.len())].to_string()),
        ])
        .style(style)
        .height(1)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(14),
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Length(20),
            Constraint::Length(8),
            Constraint::Min(20),
        ],
    )
    .header(header)
    .block(Block::default()
        .borders(Borders::ALL)
        .title(format!("Containers ({}) - Enter:Browse Files", app.containers.len())));

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
            Cell::from(img.id[..12.min(img.id.len())].to_string()),
            Cell::from(img.tags.clone()),
            Cell::from(img.size.clone()),
            Cell::from(img.created[..19.min(img.created.len())].to_string()),
        ])
        .style(style)
        .height(1)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(14),
            Constraint::Min(30),
            Constraint::Length(12),
            Constraint::Length(20),
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
            Cell::from(n.id[..12.min(n.id.len())].to_string()),
            Cell::from(n.name.clone()),
            Cell::from(n.driver.clone()),
            Cell::from(n.subnet.clone()),
        ])
        .style(style)
        .height(1)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(14),
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Min(20),
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
            Cell::from(v.name.clone()),
            Cell::from(v.driver.clone()),
            Cell::from(v.mountpoint.clone()),
        ])
        .style(style)
        .height(1)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Min(40),
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

        let icon = if file.is_dir { "📁" } else { "📄" };
        let size = if file.is_dir {
            String::new()
        } else {
            format_size(file.size)
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", icon), style),
            Span::styled(file.name.clone(), style),
            Span::styled(format!("  {}", size), Style::default().fg(Color::Gray)),
        ])));
    }

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("Files: {}", app.current_path)));

    f.render_widget(list, chunks[0]);

    let preview = if let Some(file) = app.files.get(app.selected_index) {
        if file.is_dir {
            "Directory".to_string()
        } else {
            let rootfs = app.data_dir.join("containers")
                .join(app.selected_container.as_deref().unwrap_or(""))
                .join("rootfs");
            let full_path = rootfs.join(file.path.trim_start_matches('/'));

            if let Ok(content) = std::fs::read_to_string(&full_path) {
                if content.len() > 5000 {
                    format!("{}...\n\n[File too large, showing first 5000 chars]", &content[..5000])
                } else {
                    content
                }
            } else {
                "Cannot read file".to_string()
            }
        }
    } else {
        "Select a file to preview".to_string()
    };

    let preview_widget = Paragraph::new(preview)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Preview"))
        .style(Style::default().fg(Color::White));

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
                Span::styled(line.to_string(), Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let editor = Paragraph::new(lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(if app.editor_modified {
                "Editor *modified*"
            } else {
                "Editor"
            }))
        .style(Style::default().fg(Color::White));

    f.render_widget(editor, chunks[0]);

    let status = format!(
        "Ln {}, Col {} | Ctrl+S: Save | Esc: Close",
        app.editor_cursor_y + 1,
        app.editor_cursor_x + 1
    );

    let status_bar = Paragraph::new(status)
        .style(Style::default().fg(Color::DarkGray).bg(Color::Black));

    f.render_widget(status_bar, chunks[1]);
}

fn draw_marketplace(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("ID"),
        Cell::from("NAME"),
        Cell::from("VERSION"),
        Cell::from("CATEGORY"),
        Cell::from("STATUS"),
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
            Cell::from(ext.id.clone()),
            Cell::from(ext.name.clone()),
            Cell::from(ext.version.clone()),
            Cell::from(ext.category.clone()),
            Cell::from(status),
        ])
        .style(style)
        .height(1)
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(30),
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(Block::default()
        .borders(Borders::ALL)
        .title(format!("Marketplace ({})", app.marketplace.len())));

    f.render_widget(table, area);
}

fn draw_logs(f: &mut Frame, app: &App, area: Rect) {
    let logs: Vec<Line> = app.logs.iter().map(|l| {
        Line::from(Span::styled(l.clone(), Style::default().fg(Color::White)))
    }).collect();

    let paragraph = Paragraph::new(logs)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Logs"));

    f.render_widget(paragraph, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.mode {
        AppMode::Normal => {
            match app.active_tab {
                super::app::ActiveTab::Marketplace => "q:Quit | Tab:Switch | ↑↓:Navigate | u/Enter:Uninstall | r:Refresh | h:Help",
                super::app::ActiveTab::Containers => "q:Quit | Tab:Switch | ↑↓:Navigate | Enter:Browse Files | r:Refresh | h:Help",
                _ => "q:Quit | Tab:Switch | ↑↓:Navigate | r:Refresh | h:Help",
            }
        },
        AppMode::ContainerFiles => "q:Back | ↑↓:Navigate | Enter:Open | d:Delete | n:NewFile | m:NewDir | Esc:Exit",
        AppMode::FileEditor => "Ctrl+S:Save | Esc:Close",
        AppMode::CommandInput => "Enter:Confirm | Esc:Cancel",
        AppMode::ConfirmDelete => "y:Yes | n:No",
    };

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(app.status_message.clone(), Style::default().fg(Color::Yellow)),
        Span::raw("  |  "),
        Span::styled(help_text, Style::default().fg(Color::Gray)),
    ]))
    .block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}

fn draw_confirm_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(40, 20, f.area());

    let text = vec![
        Line::from(Span::styled(
            app.confirm_message.clone(),
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
            app.status_message.clone(),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Cyan)),
            Span::styled(app.command_input.clone(), Style::default().fg(Color::White)),
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
    let area = centered_rect(60, 60, f.area());

    let help_text = vec![
        Line::from(Span::styled("Qcker TUI Help", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Tab / Shift+Tab  Switch between tabs"),
        Line::from("  ↑ / k            Move up"),
        Line::from("  ↓ / j            Move down"),
        Line::from(""),
        Line::from(Span::styled("Container Actions:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Enter            Browse container files"),
        Line::from("  s                Start selected container"),
        Line::from("  x                Stop selected container"),
        Line::from("  d                Delete selected container"),
        Line::from(""),
        Line::from(Span::styled("Marketplace:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  u / Enter        Uninstall selected extension"),
        Line::from("  r                Refresh extension list"),
        Line::from("  Request new:     github.com/qcker/qcker-extensions/issues"),
        Line::from(""),
        Line::from(Span::styled("File Browser:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Enter            Open file/directory"),
        Line::from("  Backspace        Go to parent directory"),
        Line::from("  e                Edit file in editor"),
        Line::from("  d                Delete file/directory"),
        Line::from("  n                Create new file"),
        Line::from("  m                Create new directory"),
        Line::from("  u                Upload file from host"),
        Line::from("  q / Esc          Exit file browser"),
        Line::from(""),
        Line::from(Span::styled("Editor:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+S           Save file"),
        Line::from("  Esc              Close editor"),
        Line::from(""),
        Line::from(Span::styled("General:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  r                Refresh data"),
        Line::from("  h                Toggle this help"),
        Line::from("  q / Esc          Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Help")
            .title_style(Style::default().fg(Color::Cyan))
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
