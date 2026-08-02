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
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());
    draw_header(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);
    match app.mode {
        AppMode::ConfirmDelete => draw_confirm_popup(f, app),
        AppMode::NewContainer => draw_new_container_popup(f, app),
        AppMode::ImagePull => draw_pull_image_popup(f, app),
        _ => {}
    }
    if app.show_help { draw_help_popup(f); }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Containers", "Images", "Networks", "Volumes", "Stats", "Extensions", "Logs"];
    let selected = match app.active_tab {
        ActiveTab::Containers => 0, ActiveTab::Images => 1, ActiveTab::Networks => 2,
        ActiveTab::Volumes => 3, ActiveTab::Stats => 4, ActiveTab::Extensions => 5,
        ActiveTab::Logs => 6,
    };
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Qcker Dashboard "))
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
        ActiveTab::Stats => draw_stats(f, app, area),
        ActiveTab::Extensions => draw_extensions(f, app, area),
        ActiveTab::Logs => draw_logs(f, app, area),
    }
}

fn draw_containers(f: &mut Frame, app: &App, area: Rect) {
    let sel = app.selected_action;
    let actions = ["NEW","START","STOP","DEL","EXEC","LOGS"];
    let mut btn_parts: Vec<Span> = Vec::new();
    btn_parts.push(Span::raw("["));
    for (i, name) in actions.iter().enumerate() {
        let is_sel = i == sel;
        let fg = if is_sel { Color::Black } else { Color::White };
        let bg = if is_sel { Color::Yellow } else { Color::DarkGray };
        btn_parts.push(Span::styled(name.to_string(), Style::default().fg(fg).bg(bg)));
        btn_parts.push(Span::raw("] "));
    }
    let btn_para = Paragraph::new(Line::from(btn_parts))
        .style(Style::default().bg(Color::Blue));
    f.render_widget(btn_para, area);

    let header = Row::new(vec!["ID","NAME","STATUS","IMAGE","PID","CREATED"])
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)).height(1);

    let rows: Vec<Row> = app.containers.iter().enumerate().skip(app.scroll_offset).map(|(i, c)| {
        let style = if i == app.selected_index {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else { Style::default().fg(Color::White) };
        let ss = match c.status.as_str() {
            "running" => Style::default().fg(Color::Green),
            "created" => Style::default().fg(Color::Yellow),
            "stopped" => Style::default().fg(Color::Red),
            _ => Style::default().fg(Color::Gray),
        };
        Row::new(vec![
            Cell::from(truncate(&c.id, 12)), Cell::from(truncate(&c.name, 18)),
            Cell::from(Span::styled(truncate(&c.status, 10), ss)),
            Cell::from(truncate(&c.image, 18)),
            Cell::from(c.pid.map_or("-".to_string(), |p| p.to_string())),
            Cell::from(truncate(&c.created, 16)),
        ]).style(style).height(1)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Length(13), Constraint::Length(19), Constraint::Length(11),
        Constraint::Length(19), Constraint::Length(8), Constraint::Min(17),
    ]).header(header)
    .block(Block::default().borders(Borders::ALL)
        .title(format!(" Containers ({}) ", app.containers.len())));
    f.render_widget(table, area);
}

fn draw_images(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["ID","TAGS","SIZE","CREATED"])
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)).height(1);
    let rows: Vec<Row> = app.images.iter().enumerate().skip(app.scroll_offset).map(|(i, img)| {
        let style = if i == app.selected_index { Style::default().bg(Color::DarkGray).fg(Color::White) }
        else { Style::default().fg(Color::White) };
        Row::new(vec![
            Cell::from(truncate(&img.id, 12)), Cell::from(truncate(&img.tags, 30)),
            Cell::from(truncate(&img.size, 10)), Cell::from(truncate(&img.created, 16)),
        ]).style(style).height(1)
    }).collect();
    let table = Table::new(rows, [
        Constraint::Length(13), Constraint::Min(31), Constraint::Length(11), Constraint::Length(17),
    ]).header(header).block(Block::default().borders(Borders::ALL)
        .title(format!(" Images ({}) ", app.images.len())));
    f.render_widget(table, area);
}

fn draw_networks(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["ID","NAME","DRIVER","SUBNET"])
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)).height(1);
    let rows: Vec<Row> = app.networks.iter().enumerate().skip(app.scroll_offset).map(|(i, n)| {
        let style = if i == app.selected_index { Style::default().bg(Color::DarkGray).fg(Color::White) }
        else { Style::default().fg(Color::White) };
        Row::new(vec![
            Cell::from(truncate(&n.id, 12)), Cell::from(truncate(&n.name, 18)),
            Cell::from(truncate(&n.driver, 10)), Cell::from(truncate(&n.subnet, 18)),
        ]).style(style).height(1)
    }).collect();
    let table = Table::new(rows, [
        Constraint::Length(13), Constraint::Length(19), Constraint::Length(11), Constraint::Min(19),
    ]).header(header).block(Block::default().borders(Borders::ALL)
        .title(format!(" Networks ({}) ", app.networks.len())));
    f.render_widget(table, area);
}

fn draw_volumes(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["NAME","DRIVER","MOUNTPOINT"])
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)).height(1);
    let rows: Vec<Row> = app.volumes.iter().enumerate().skip(app.scroll_offset).map(|(i, v)| {
        let style = if i == app.selected_index { Style::default().bg(Color::DarkGray).fg(Color::White) }
        else { Style::default().fg(Color::White) };
        Row::new(vec![
            Cell::from(truncate(&v.name, 18)), Cell::from(truncate(&v.driver, 10)),
            Cell::from(truncate(&v.mountpoint, 40)),
        ]).style(style).height(1)
    }).collect();
    let table = Table::new(rows, [
        Constraint::Length(19), Constraint::Length(11), Constraint::Min(41),
    ]).header(header).block(Block::default().borders(Borders::ALL)
        .title(format!(" Volumes ({}) ", app.volumes.len())));
    f.render_widget(table, area);
}

fn draw_stats(f: &mut Frame, app: &App, area: Rect) {
    let s = &app.system_stats;
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled("System Overview", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("CPU: ", Style::default().fg(Color::Yellow)),
            Span::styled(format!("{:.1}%", s.cpu_percent), Style::default().fg(Color::White)),
            Span::styled("  Memory: ", Style::default().fg(Color::Yellow)),
            Span::styled(format!("{:.0}MB / {:.0}MB ({:.1}%)", s.mem_used_mb, s.mem_total_mb, s.mem_percent), Style::default().fg(Color::White)),
        ]),
    ];
    let ups = App::format_uptime(s.uptime_secs);
    lines.push(Line::from(vec![
        Span::styled("Load Avg: ", Style::default().fg(Color::Yellow)),
        Span::styled(format!("{:.2} {:.2} {:.2}", s.load_avg[0], s.load_avg[1], s.load_avg[2]), Style::default().fg(Color::White)),
        Span::styled("  Uptime: ", Style::default().fg(Color::Yellow)),
        Span::styled(&ups, Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Container Summary", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Running: ", Style::default().fg(Color::Green)),
        Span::styled(format!("{}", s.running), Style::default().fg(Color::Green)),
        Span::styled("  Stopped: ", Style::default().fg(Color::Red)),
        Span::styled(format!("{}", s.stopped), Style::default().fg(Color::Red)),
        Span::styled("  Images: ", Style::default().fg(Color::Yellow)),
        Span::styled(format!("{}", s.total_images), Style::default().fg(Color::Yellow)),
        Span::styled("  Volumes: ", Style::default().fg(Color::Cyan)),
        Span::styled(format!("{}", s.total_volumes), Style::default().fg(Color::Cyan)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Per-Container Stats", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
    lines.push(Line::from(""));

    if app.containers.is_empty() {
        lines.push(Line::from(Span::styled("  No containers running", Style::default().fg(Color::Gray))));
    } else {
        for c in &app.containers.iter().take(10).collect::<Vec<_>>() {
            let st = app.container_stats.get(&c.id);
            let cpu = st.map(|s| format!("{:.1}%", s.cpu_percent)).unwrap_or("-".to_string());
            let mem = st.map(|s| format!("{:.1} MB", s.memory_mb)).unwrap_or("-".to_string());
            let pids = st.map(|s| s.pids.to_string()).unwrap_or("-".to_string());
            let color = match c.status.as_str() { "running" => Color::Green, "paused" => Color::Yellow, _ => Color::Red };
            lines.push(Line::from(vec![
                Span::styled(&c.name, Style::default().fg(Color::White)),
                Span::styled(format!("  CPU: {}  Mem: {}  PIDs: {}", cpu, mem, pids), Style::default().fg(color)),
            ]));
        }
        if app.containers.len() > 10 {
            lines.push(Line::from(Span::styled(format!("  ... and {} more", app.containers.len() - 10), Style::default().fg(Color::Gray))));
        }
    }

    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" System Stats — {} containers ", app.containers.len())))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(para, area);
}

fn draw_extensions(f: &mut Frame, app: &App, area: Rect) {
    use super::app::MarketplaceExt;
    let exts = &app.extensions;

    // Action bar
    let sel = app.selected_action;
    let actions = ["INST","ENBL","DSBL","UNST"];
    let mut btn_parts: Vec<Span> = Vec::new();
    btn_parts.push(Span::raw("["));
    for (i, name) in actions.iter().enumerate() {
        let is_sel = i == sel;
        let fg = if is_sel { Color::Black } else { Color::White };
        let bg = if is_sel { Color::Yellow } else { Color::DarkGray };
        btn_parts.push(Span::styled(name.to_string(), Style::default().fg(fg).bg(bg)));
        btn_parts.push(Span::raw("] "));
    }
    let btn_para = Paragraph::new(Line::from(btn_parts))
        .style(Style::default().bg(Color::Blue));
    f.render_widget(btn_para, area);

    let header = Row::new(vec!["NAME","VERSION","CATEGORY","STATUS","DESCRIPTION"])
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)).height(1);

    let rows: Vec<Row> = exts.iter().enumerate().skip(app.scroll_offset).map(|(i, ext)| {
        let style = if i == app.selected_index {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else { Style::default().fg(Color::White) };
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
        ]).style(style).height(1)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Length(17), Constraint::Length(9), Constraint::Length(11),
        Constraint::Length(11), Constraint::Min(31),
    ]).header(header)
    .block(Block::default().borders(Borders::ALL)
        .title(format!(" Extensions ({}) ", exts.len())));
    f.render_widget(table, area);
}

fn draw_logs(f: &mut Frame, app: &App, area: Rect) {
    let logs: Vec<Line> = app.logs.iter().skip(app.scroll_offset).map(|l| {
        Line::from(Span::styled(truncate(l, 120), Style::default().fg(Color::White)))
    }).collect();
    let para = Paragraph::new(logs)
        .block(Block::default().borders(Borders::ALL).title(" Logs "))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(para, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.mode {
        AppMode::Normal => match app.active_tab {
            ActiveTab::Containers => "n:New ENTER:Action [sxdiw]:Keys Tab:Switch r:Refresh h:Help q:Quit",
            ActiveTab::Images => "p:Pull Tab:Switch r:Refresh h:Help q:Quit",
            ActiveTab::Extensions => "i:Install e:Enable d:Disable u:Uninstall ENTER:Action [ieDu]:Keys Tab:Switch r:Refresh h:Help q:Quit",
            _ => "Tab:Switch r:Refresh h:Help q:Quit",
        },
        AppMode::ConfirmDelete => "y:Yes n:No Esc:Cancel",
        AppMode::NewContainer => "Tab:NextField Enter:Create Esc:Cancel",
        AppMode::ExecCommand => "Enter:Run Esc:Cancel",
        AppMode::ImagePull => "Enter:Pull Esc:Cancel",
        AppMode::WatchingLogs => "Esc/q:Stop r:Refresh g/G:Scroll",
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
    let area = centered_rect(40, 15, f.area());
    let text = vec![
        Line::from(Span::styled(truncate(&app.confirm_message, 30),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("y = Yes, n = No, Esc = Cancel", Style::default().fg(Color::Gray))),
    ];
    let popup = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Confirm")
            .style(Style::default().bg(Color::DarkGray)))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(Clear, area);
    f.render_widget(popup, area);
}

fn draw_new_container_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 20, f.area());
    let text = vec![
        Line::from(Span::styled("Create New Container", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![Span::styled("Name:     ", Style::default().fg(Color::Yellow)), Span::styled(&app.new_name, Style::default().fg(Color::White))]),
        Line::from(vec![Span::styled("Image:    ", Style::default().fg(Color::Yellow)), Span::styled(&app.new_image, Style::default().fg(Color::White))]),
        Line::from(vec![Span::styled("Command:  ", Style::default().fg(Color::Yellow)), Span::styled(&app.new_cmd, Style::default().fg(Color::White))]),
        Line::from(""),
        Line::from(Span::styled("Tab: switch field | Enter: create | Esc: cancel", Style::default().fg(Color::Gray))),
    ];
    let popup = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("New Container")
            .style(Style::default().bg(Color::DarkGray)));
    f.render_widget(Clear, area);
    f.render_widget(popup, area);
}

fn draw_pull_image_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(40, 10, f.area());
    let text = vec![
        Line::from(Span::styled("Pull Image", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![Span::styled("Image: ", Style::default().fg(Color::Yellow)), Span::styled(&app.pull_input, Style::default().fg(Color::White))]),
        Line::from(""),
        Line::from(Span::styled("Enter: pull | Esc: cancel", Style::default().fg(Color::Gray))),
    ];
    let popup = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Pull Image")
            .style(Style::default().bg(Color::DarkGray)));
    f.render_widget(Clear, area);
    f.render_widget(popup, area);
}

fn draw_help_popup(f: &mut Frame) {
    let area = centered_rect(60, 55, f.area());
    let popup = Paragraph::new(vec![
        Line::from(Span::styled("Qcker Dashboard v0.1.0", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("  Tab/BackTab  Switch tabs (7 tabs)"),
        Line::from("  j/k or arrows  Navigate lists"),
        Line::from("  Enter  Select / Execute action"),
        Line::from("  Left/Right  Move action focus (Containers tab)"),
        Line::from("  Containers: n:s:x:d:i:w  Extensions: i:e:d:u"),
        Line::from("  p  Pull image   ←→  Move action focus   Enter  Execute"),
        Line::from("  r  Refresh   h  Toggle help   q  Quit"),
        Line::from(""),
        Line::from(Span::styled("Mouse: Click tabs, rows, buttons, scroll wheel", Style::default().fg(Color::Gray))),
    ])
    .block(Block::default().borders(Borders::ALL).title("Help"))
    .style(Style::default().bg(Color::DarkGray));
    f.render_widget(Clear, area);
    f.render_widget(popup, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let pl = Layout::default()
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
        .split(pl[1])[1]
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len { s.to_string() }
    else { format!("{}...", s.chars().take(max_len.saturating_sub(3)).collect::<String>()) }
}
