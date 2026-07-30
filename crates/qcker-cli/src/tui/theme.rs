use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub bg: Color,
    pub text: Color,
    pub text_dim: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
    pub border: Color,
    pub border_active: Color,
    pub tab_active: Style,
    pub tab_inactive: Style,
    pub table_header: Style,
    pub table_row: Style,
    pub table_selected: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: Color::Reset,
            text: Color::White,
            text_dim: Color::DarkGray,
            accent: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
            selected_bg: Color::DarkGray,
            selected_fg: Color::White,
            border: Color::DarkGray,
            border_active: Color::Cyan,
            tab_active: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            tab_inactive: Style::default().fg(Color::DarkGray),
            table_header: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            table_row: Style::default().fg(Color::White),
            table_selected: Style::default().fg(Color::White).bg(Color::DarkGray),
        }
    }
}

impl Theme {
    pub fn dracula() -> Self {
        Self {
            bg: Color::Rgb(40, 42, 54),
            text: Color::Rgb(248, 248, 242),
            text_dim: Color::Rgb(98, 114, 164),
            accent: Color::Rgb(189, 147, 249),
            success: Color::Rgb(80, 250, 123),
            warning: Color::Rgb(241, 250, 140),
            error: Color::Rgb(255, 85, 85),
            info: Color::Rgb(139, 233, 253),
            selected_bg: Color::Rgb(68, 71, 90),
            selected_fg: Color::Rgb(248, 248, 242),
            border: Color::Rgb(68, 71, 90),
            border_active: Color::Rgb(189, 147, 249),
            tab_active: Style::default().fg(Color::Rgb(189, 147, 249)).add_modifier(Modifier::BOLD),
            tab_inactive: Style::default().fg(Color::Rgb(98, 114, 164)),
            table_header: Style::default().fg(Color::Rgb(189, 147, 249)).add_modifier(Modifier::BOLD),
            table_row: Style::default().fg(Color::Rgb(248, 248, 242)),
            table_selected: Style::default().fg(Color::Rgb(248, 248, 242)).bg(Color::Rgb(68, 71, 90)),
        }
    }
}
