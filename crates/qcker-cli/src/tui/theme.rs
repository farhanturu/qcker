use ratatui::style::{Color, Modifier, Style};

pub const BG: Color = Color::Rgb(28, 30, 38);
pub const SURFACE: Color = Color::Rgb(40, 42, 54);
pub const SURFACE_BRIGHT: Color = Color::Rgb(55, 57, 73);

pub const TEXT: Color = Color::Rgb(248, 248, 242);
pub const TEXT_DIM: Color = Color::Rgb(98, 114, 164);
pub const TEXT_SUBTLE: Color = Color::Rgb(166, 173, 200);

pub const CYAN: Color = Color::Rgb(139, 233, 253);
pub const ACCENT: Color = CYAN;
pub const PURPLE: Color = Color::Rgb(189, 147, 249);
pub const GREEN: Color = Color::Rgb(80, 250, 123);
pub const YELLOW: Color = Color::Rgb(241, 250, 140);
pub const RED: Color = Color::Rgb(255, 85, 85);
pub const ORANGE: Color = Color::Rgb(255, 184, 108);
pub const PINK: Color = Color::Rgb(255, 121, 198);

pub const BORDER: Color = Color::Rgb(68, 71, 90);
pub const SELECTED_BG: Color = Color::Rgb(68, 71, 90);
pub const HEADER_BG: Color = Color::Rgb(33, 34, 44);
pub const FOOTER_BG: Color = Color::Rgb(33, 34, 44);

pub const STATUS_RUNNING: Color = Color::Rgb(80, 250, 123);
pub const STATUS_STOPPED: Color = Color::Rgb(255, 85, 85);
pub const STATUS_CREATED: Color = Color::Rgb(241, 250, 140);
pub const STATUS_PAUSED: Color = Color::Rgb(139, 233, 253);
pub const STATUS_DEAD: Color = Color::Rgb(98, 114, 164);

pub const GAUGE_LOW: Color = Color::Rgb(80, 250, 123);
pub const GAUGE_MED: Color = Color::Rgb(241, 250, 140);
pub const GAUGE_HIGH: Color = Color::Rgb(255, 85, 85);

pub const TAB_TITLES: &[&str] = &[
    "Containers",
    "Images",
    "Networks",
    "Volumes",
    "Stats",
    "Logs",
    "Extensions",
];

pub const TAB_WIDTH: usize = 13;

pub fn text_style() -> Style {
    Style::default().fg(TEXT)
}

pub fn dim_style() -> Style {
    Style::default().fg(TEXT_DIM)
}

pub fn subtle_style() -> Style {
    Style::default().fg(TEXT_SUBTLE)
}

pub fn accent_style() -> Style {
    Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
}

pub fn purple_style() -> Style {
    Style::default().fg(PURPLE).add_modifier(Modifier::BOLD)
}

pub fn selected_style() -> Style {
    Style::default().bg(SELECTED_BG).fg(TEXT)
}

pub fn selected_accent_style() -> Style {
    Style::default().bg(SELECTED_BG).fg(CYAN).add_modifier(Modifier::BOLD)
}

pub fn header_style() -> Style {
    Style::default().bg(HEADER_BG).fg(TEXT)
}

pub fn footer_style() -> Style {
    Style::default().bg(FOOTER_BG).fg(TEXT_DIM)
}

pub fn status_style(status: &str) -> Style {
    match status {
        "Running" => Style::default().fg(STATUS_RUNNING).add_modifier(Modifier::BOLD),
        "Stopped" | "Dead" => Style::default().fg(STATUS_STOPPED),
        "Created" => Style::default().fg(STATUS_CREATED),
        "Paused" => Style::default().fg(STATUS_PAUSED),
        _ => Style::default().fg(TEXT_DIM),
    }
}

pub fn gauge_color(percent: f64) -> Color {
    if percent < 50.0 {
        GAUGE_LOW
    } else if percent < 80.0 {
        GAUGE_MED
    } else {
        GAUGE_HIGH
    }
}

pub fn border_style() -> Style {
    Style::default().fg(BORDER)
}

pub fn tab_active_style() -> Style {
    Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
}

pub fn tab_inactive_style() -> Style {
    Style::default().fg(TEXT_DIM)
}

pub fn help_style() -> Style {
    Style::default().fg(PURPLE).add_modifier(Modifier::BOLD)
}

pub fn error_style() -> Style {
    Style::default().fg(RED).add_modifier(Modifier::BOLD)
}

pub fn success_style() -> Style {
    Style::default().fg(GREEN).add_modifier(Modifier::BOLD)
}

pub fn warning_style() -> Style {
    Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)
}

pub fn info_style() -> Style {
    Style::default().fg(CYAN)
}

pub fn key_style() -> Style {
    Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)
}

pub fn block_title_style() -> Style {
    Style::default().fg(PURPLE).add_modifier(Modifier::BOLD)
}

pub fn table_header_style() -> Style {
    Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
}
