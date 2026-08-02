use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Table, Tabs},
    Frame,
};

use super::app::{ActiveTab, App, AppMode};

const BG: Color = Color::Rgb(28, 30, 38);
const SURFACE: Color = Color::Rgb(40, 42, 54);
const TEXT: Color = Color::Rgb(248, 248, 242);
const TEXT_DIM: Color = Color::Rgb(98, 114, 164);
const ACCENT: Color = Color::Rgb(139, 233, 253);
const PURPLE: Color = Color::Rgb(189, 147, 249);
const GREEN: Color = Color::Rgb(80, 250, 123);
const YELLOW: Color = Color::Rgb(241, 250, 140);
const RED: Color = Color::Rgb(255, 85, 85);
const ORANGE: Color = Color::Rgb(255, 184, 108);
const BORDER: Color = Color::Rgb(68, 71, 90);
const SELECTED_BG: Color = Color::Rgb(68, 71, 90);

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_status_bar(f, app, chunks[2]);
    draw_footer(f, app, chunks[3]);

    match app.mode {
        AppMode::ConfirmAction => draw_confirm_popup(f, app),
        AppMode::CommandInput => draw_command_popup(f, app),
        AppMode::FileEditor => draw_editor_overlay(f, app),
        _ => {}
    }

    if app.show_help {
        draw_help_popup(f, app);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = ActiveTab::all().iter().map(|tab| {
        let title = tab.title();
        if *tab == app.active_tab {
            Line::from(Span::styled(
                format!(" {} ", title),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from(Span::styled(
                format!(" {} ", title),
                Style::default().fg(TEXT_DIM),
            ))
        }
    }).collect();

    let selected = ActiveTab::all().iter().position(|t| *t == app.active_tab).unwrap_or(0);

    let tabs = Tabs::new(titles)
        .block(Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(BORDER))
            .title(Span::styled(
                " Qcker ",
                Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
            ))
            .title_alignment(ratatui::layout::Alignment::Left)
            .style(Style::default().bg(SURFACE)))
        .select(selected)
        .highlight_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
        .divider(Span::styled(" | ", Style::default().fg(BORDER)));

    f.render_widget(tabs, area);
}

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .style(Style::default().bg(BG));
    f.render_widget(block, area);

    match app.active_tab {
        ActiveTab::Containers => draw_containers(f, app, area),
        ActiveTab::Images => draw_images(f, app, area),
        ActiveTab::Networks => draw_networks(f, app, area),
        ActiveTab::Volumes => draw_volumes(f, app, area),
        ActiveTab::Stats => draw_stats(f, app, area),
        ActiveTab::Logs => draw_logs(f, app, area),
        ActiveTab::Marketplace => draw_marketplace(f, app, area),
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
    .style(Style::default().fg(PURPLE).add_modifier(Modifier::BOLD))
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app.containers.iter().enumerate().map(|(i, c)| {
        let style = if i == app.selected_index {
            Style::default().bg(SELECTED_BG).fg(TEXT)
        } else {
            Style::default().fg(TEXT)
        };

        let status_style = match c.status.as_str() {
            "running" => Style::default().fg(GREEN),
            "created" => Style::default().fg(YELLOW),
            "stopped" => Style::default().fg(RED),
            "paused" => Style::default().fg(ORANGE),
            _ => Style::default().fg(TEXT_DIM),
        };

        let status_icon = match c.status.as_str() {
            "running" => "+",
            "created" => "~",
            "stopped" => "-",
            "paused" => "||",
            _ => "?",
        };

        Row::new(vec![
            Cell::from(truncate(&c.id, 12)),
            Cell::from(truncate(&c.name, 18)),
            Cell::from(Span::styled(
                format!("{} {}", status_icon, truncate(&c.status, 10)),
                status_style,
            )),
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
            Constraint::Length(13),
            Constraint::Length(19),
            Constraint::Length(8),
            Constraint::Min(17),
        ],
    )
    .header(header)
    .block(Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(
            format!(" Containers ({}) ", app.containers.len()),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(BG)))
    .highlight_style(Style::default().bg(SELECTED_BG));

    f.render_widget(table, area);
}

fn draw_images(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("ID"),
        Cell::from("TAGS"),
        Cell::from("SIZE"),
        Cell::from("CREATED"),
    ])
    .style(Style::default().fg(PURPLE).add_modifier(Modifier::BOLD))
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app.images.iter().enumerate().map(|(i, img)| {
        let style = if i == app.selected_index {
            Style::default().bg(SELECTED_BG).fg(TEXT)
        } else {
            Style::default().fg(TEXT)
        };

        Row::new(vec![
            Cell::from(truncate(&img.id, 12)),
            Cell::from(truncate(&img.tags, 30)),
            Cell::from(Span::styled(truncate(&img.size, 10), Style::default().fg(YELLOW))),
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
        .borders(Borders::NONE)
        .title(Span::styled(
            format!(" Images ({}) ", app.images.len()),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(BG)))
    .highlight_style(Style::default().bg(SELECTED_BG));

    f.render_widget(table, area);
}

fn draw_networks(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("ID"),
        Cell::from("NAME"),
        Cell::from("DRIVER"),
        Cell::from("SUBNET"),
    ])
    .style(Style::default().fg(PURPLE).add_modifier(Modifier::BOLD))
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app.networks.iter().enumerate().map(|(i, n)| {
        let style = if i == app.selected_index {
            Style::default().bg(SELECTED_BG).fg(TEXT)
        } else {
            Style::default().fg(TEXT)
        };

        Row::new(vec![
            Cell::from(truncate(&n.id, 12)),
            Cell::from(truncate(&n.name, 18)),
            Cell::from(Span::styled(truncate(&n.driver, 10), Style::default().fg(ACCENT))),
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
        .borders(Borders::NONE)
        .title(Span::styled(
            format!(" Networks ({}) ", app.networks.len()),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(BG)))
    .highlight_style(Style::default().bg(SELECTED_BG));

    f.render_widget(table, area);
}

fn draw_volumes(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("NAME"),
        Cell::from("DRIVER"),
        Cell::from("MOUNTPOINT"),
    ])
    .style(Style::default().fg(PURPLE).add_modifier(Modifier::BOLD))
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app.volumes.iter().enumerate().map(|(i, v)| {
        let style = if i == app.selected_index {
            Style::default().bg(SELECTED_BG).fg(TEXT)
        } else {
            Style::default().fg(TEXT)
        };

        Row::new(vec![
            Cell::from(truncate(&v.name, 18)),
            Cell::from(Span::styled(truncate(&v.driver, 10), Style::default().fg(ACCENT))),
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
        .borders(Borders::NONE)
        .title(Span::styled(
            format!(" Volumes ({}) ", app.volumes.len()),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(BG)))
    .highlight_style(Style::default().bg(SELECTED_BG));

    f.render_widget(table, area);
}

fn draw_stats(f: &mut Frame, app: &App, area: Rect) {
    if app.stats.is_empty() {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No running containers",
                Style::default().fg(TEXT_DIM),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Start a container to see real-time stats",
                Style::default().fg(TEXT_DIM),
            )),
        ])
        .style(Style::default().bg(BG));
        f.render_widget(msg, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(app.stats.iter().map(|_| Constraint::Length(5)).collect::<Vec<_>>())
        .split(area);

    for (i, stat) in app.stats.iter().enumerate() {
        if i >= chunks.len() {
            break;
        }

        let cpu_ratio = (stat.cpu_percent / 100.0).clamp(0.0, 1.0);
        let mem_ratio = if stat.memory_limit_mb > 0.0 {
            (stat.memory_mb / stat.memory_limit_mb).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let cpu_color = if cpu_ratio > 0.8 { RED } else if cpu_ratio > 0.5 { YELLOW } else { GREEN };
        let mem_color = if mem_ratio > 0.8 { RED } else if mem_ratio > 0.5 { YELLOW } else { GREEN };

        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(chunks[i]);

        let title = Paragraph::new(Line::from(vec![
            Span::styled(" ", Style::default().fg(TEXT)),
            Span::styled(truncate(&stat.name, 20), Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" ({})", truncate(&stat.id, 12)), Style::default().fg(TEXT_DIM)),
        ]))
        .style(Style::default().bg(BG));
        f.render_widget(title, inner[0]);

        let cpu_gauge = Gauge::default()
            .block(Block::default().style(Style::default().bg(BG)))
            .gauge_style(Style::default().fg(cpu_color).bg(SURFACE))
            .ratio(cpu_ratio)
            .label(Span::styled(
                format!("CPU: {:.1}%", stat.cpu_percent),
                Style::default().fg(TEXT),
            ));
        f.render_widget(cpu_gauge, inner[1]);

        let mem_gauge = Gauge::default()
            .block(Block::default().style(Style::default().bg(BG)))
            .gauge_style(Style::default().fg(mem_color).bg(SURFACE))
            .ratio(mem_ratio)
            .label(Span::styled(
                format!("MEM: {:.0}/{:.0} MB", stat.memory_mb, stat.memory_limit_mb),
                Style::default().fg(TEXT),
            ));
        f.render_widget(mem_gauge, inner[2]);

        let info = Paragraph::new(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(format!("PIDs: {} ", stat.pids), Style::default().fg(TEXT_DIM)),
            Span::styled(format!("NET: {}↓ {}↑ ", format_bytes(stat.net_rx), format_bytes(stat.net_tx)), Style::default().fg(ACCENT)),
            Span::styled(format!("IO: {}↓ {}↑", format_bytes(stat.block_rx), format_bytes(stat.block_tx)), Style::default().fg(YELLOW)),
        ]))
        .style(Style::default().bg(BG));
        f.render_widget(info, inner[3]);
    }
}

fn draw_logs(f: &mut Frame, app: &App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.logs.len();
    let start = app.scroll_offset.min(total.saturating_sub(visible_height));
    let end = (start + visible_height).min(total);

    let logs: Vec<Line> = app.logs[start..end].iter().map(|l| {
        let level_style = match l.level.as_str() {
            "ERROR" | "ERR" => Style::default().fg(RED),
            "WARN" | "WARNING" => Style::default().fg(YELLOW),
            "INFO" => Style::default().fg(ACCENT),
            "DEBUG" | "TRACE" => Style::default().fg(TEXT_DIM),
            _ => Style::default().fg(TEXT),
        };

        Line::from(vec![
            Span::styled(format!(" {} ", l.timestamp), Style::default().fg(TEXT_DIM)),
            Span::styled(format!("{:5} ", l.level), level_style),
            Span::styled(truncate(&l.message, 120), Style::default().fg(TEXT)),
        ])
    }).collect();

    let paragraph = Paragraph::new(logs)
        .block(Block::default()
            .borders(Borders::NONE)
            .title(Span::styled(
                format!(" Logs ({}) ", total),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(BG)));

    f.render_widget(paragraph, area);
}

fn draw_marketplace(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("NAME"),
        Cell::from("VERSION"),
        Cell::from("CATEGORY"),
        Cell::from("STATUS"),
        Cell::from("DESCRIPTION"),
    ])
    .style(Style::default().fg(PURPLE).add_modifier(Modifier::BOLD))
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app.marketplace.iter().enumerate().map(|(i, ext)| {
        let style = if i == app.selected_index {
            Style::default().bg(SELECTED_BG).fg(TEXT)
        } else {
            Style::default().fg(TEXT)
        };

        let status = if ext.built_in {
            Span::styled("Built-in", Style::default().fg(GREEN))
        } else if ext.installed {
            Span::styled("Installed", Style::default().fg(ACCENT))
        } else {
            Span::styled("Available", Style::default().fg(YELLOW))
        };

        Row::new(vec![
            Cell::from(truncate(&ext.name, 16)),
            Cell::from(Span::styled(truncate(&ext.version, 8), Style::default().fg(TEXT_DIM))),
            Cell::from(Span::styled(truncate(&ext.category, 10), Style::default().fg(ACCENT))),
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
        .borders(Borders::NONE)
        .title(Span::styled(
            format!(" Extensions ({}) ", app.marketplace.len()),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(BG)))
    .highlight_style(Style::default().bg(SELECTED_BG));

    f.render_widget(table, area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let left = match app.mode {
        AppMode::ContainerFiles => {
            format!(" Files: {} ", app.current_path)
        }
        _ => {
            format!(" {} ", app.status_message)
        }
    };

    let right = if app.auto_refresh {
        format!("Auto:ON | {} ", app.last_refresh)
    } else {
        "Auto:OFF ".to_string()
    };

    let status = Paragraph::new(Line::from(vec![
        Span::styled(left, Style::default().fg(YELLOW).bg(SURFACE)),
        Span::styled(right, Style::default().fg(TEXT_DIM).bg(SURFACE)),
    ]))
    .style(Style::default().bg(SURFACE));

    f.render_widget(status, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.mode {
        AppMode::Normal => {
            match app.active_tab {
                ActiveTab::Containers => "q:Quit Tab:Switch Enter:Files s:Stop d:Delete r:Refresh a:AutoRefresh h:Help",
                ActiveTab::Marketplace => "q:Quit Tab:Switch u:Uninstall r:Refresh h:Help",
                ActiveTab::Logs => "q:Quit Tab:Switch g:Top G:Bottom r:Refresh h:Help",
                ActiveTab::Stats => "q:Quit Tab:Switch r:Refresh h:Help",
                _ => "q:Quit Tab:Switch r:Refresh a:AutoRefresh h:Help",
            }
        },
        AppMode::ContainerFiles => "Esc:Back Enter:Open e:Edit d:Delete n:New m:Mkdir r:Refresh",
        AppMode::FileEditor => "Ctrl+S:Save Esc:Close Arrow:Move",
        AppMode::CommandInput => "Enter:Confirm Esc:Cancel",
        AppMode::ConfirmAction => "y:Yes n:No",
    };

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(help_text, Style::default().fg(TEXT_DIM).bg(SURFACE)),
    ]))
    .style(Style::default().bg(SURFACE));

    f.render_widget(footer, area);
}

fn draw_editor_overlay(f: &mut Frame, app: &App) {
    let area = centered_rect(80, 80, f.area());

    let lines: Vec<Line> = app.editor_content
        .lines()
        .enumerate()
        .map(|(i, line)| {
            let line_num = format!("{:4} ", i + 1);
            Line::from(vec![
                Span::styled(line_num, Style::default().fg(TEXT_DIM).bg(SURFACE)),
                Span::styled(truncate(line, 200), Style::default().fg(TEXT)),
            ])
        })
        .collect();

    let title = if app.editor_modified {
        " Editor * "
    } else {
        " Editor "
    };

    let editor = Paragraph::new(lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(PURPLE))
            .title(Span::styled(title, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
            .style(Style::default().bg(BG)))
        .wrap(ratatui::widgets::Wrap { trim: false });

    f.render_widget(Clear, area);
    f.render_widget(editor, area);

    let status = format!(
        " Ln {} Col {} | Ctrl+S:Save Esc:Close",
        app.editor_cursor_y + 1,
        app.editor_cursor_x + 1
    );

    let status_bar = Paragraph::new(Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(status, Style::default().fg(TEXT_DIM).bg(SURFACE)),
    ]))
    .style(Style::default().bg(SURFACE));

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    f.render_widget(status_bar, inner[1]);
}

fn draw_confirm_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(40, 20, f.area());

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", truncate(&app.confirm_message, 40)),
            Style::default().fg(RED).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  y = Yes, n = No",
            Style::default().fg(TEXT_DIM),
        )),
    ];

    let popup = Paragraph::new(text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(RED))
            .title(Span::styled(" Confirm ", Style::default().fg(RED).add_modifier(Modifier::BOLD)))
            .style(Style::default().bg(SURFACE)))
        .alignment(ratatui::layout::Alignment::Left);

    f.render_widget(Clear, area);
    f.render_widget(popup, area);
}

fn draw_command_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 15, f.area());

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", truncate(&app.status_message, 40)),
            Style::default().fg(YELLOW),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  > ", Style::default().fg(ACCENT)),
            Span::styled(truncate(&app.command_input, 30), Style::default().fg(TEXT)),
            Span::styled("_", Style::default().fg(TEXT).add_modifier(Modifier::SLOW_BLINK)),
        ]),
    ];

    let popup = Paragraph::new(text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ACCENT))
            .title(Span::styled(" Input ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
            .style(Style::default().bg(SURFACE)));

    f.render_widget(Clear, area);
    f.render_widget(popup, area);
}

fn draw_help_popup(f: &mut Frame, _app: &App) {
    let area = centered_rect(50, 60, f.area());

    let help_text = vec![
        Line::from(Span::styled("  Qcker TUI Help", Style::default().fg(PURPLE).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("  Navigation", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("  Tab / Shift+Tab   Switch tabs", Style::default().fg(TEXT))),
        Line::from(Span::styled("  Up/Down or j/k    Navigate items", Style::default().fg(TEXT))),
        Line::from(Span::styled("  PageUp/PageDown   Scroll pages", Style::default().fg(TEXT))),
        Line::from(Span::styled("  Mouse click       Select item", Style::default().fg(TEXT))),
        Line::from(Span::styled("  Mouse scroll      Scroll list", Style::default().fg(TEXT))),
        Line::from(""),
        Line::from(Span::styled("  Containers", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("  Enter             Browse files", Style::default().fg(TEXT))),
        Line::from(Span::styled("  s                 Stop container", Style::default().fg(TEXT))),
        Line::from(Span::styled("  d                 Delete container", Style::default().fg(TEXT))),
        Line::from(""),
        Line::from(Span::styled("  Files", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("  Enter             Open file/dir", Style::default().fg(TEXT))),
        Line::from(Span::styled("  e                 Edit file", Style::default().fg(TEXT))),
        Line::from(Span::styled("  d                 Delete file", Style::default().fg(TEXT))),
        Line::from(Span::styled("  n                 New file", Style::default().fg(TEXT))),
        Line::from(Span::styled("  m                 New directory", Style::default().fg(TEXT))),
        Line::from(Span::styled("  Backspace         Go up", Style::default().fg(TEXT))),
        Line::from(""),
        Line::from(Span::styled("  Editor", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("  Ctrl+S            Save", Style::default().fg(TEXT))),
        Line::from(Span::styled("  Esc               Close", Style::default().fg(TEXT))),
        Line::from(Span::styled("  Arrow keys        Move cursor", Style::default().fg(TEXT))),
        Line::from(""),
        Line::from(Span::styled("  General", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("  r                 Refresh", Style::default().fg(TEXT))),
        Line::from(Span::styled("  a                 Toggle auto-refresh", Style::default().fg(TEXT))),
        Line::from(Span::styled("  h                 Help", Style::default().fg(TEXT))),
        Line::from(Span::styled("  q                 Quit", Style::default().fg(TEXT))),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(PURPLE))
            .title(Span::styled(" Help ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
            .style(Style::default().bg(SURFACE)));

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
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max_len.saturating_sub(3)).collect::<String>())
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
