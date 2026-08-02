use ratatui::style::{Color, Modifier, Style};

// ── Dracula Theme for Qcker TUI ──────────────────────────────────────
// Official Dracula color palette adapted for terminal UI usage.
// See: https://draculatheme.com/contribute
//
// This module provides both raw color constants for flexible use and
// pre-built Style helpers for common UI patterns.

// ── Background ──────────────────────────────────────────────────────
#[allow(dead_code)]
pub const BG: Color = Color::Rgb(28, 30, 38);          // #1E1E2E (Catppuccin Mocha base)
pub const SURFACE: Color = Color::Rgb(40, 42, 54);     // #282A36 (Dracula Background)
pub const SURFACE_BRIGHT: Color = Color::Rgb(55, 57, 73); // #373949 (Elevated surface)

// ── Foreground / Text ───────────────────────────────────────────────
pub const TEXT: Color = Color::Rgb(248, 248, 242);     // #F8F8F2 (Dracula Foreground)
pub const TEXT_DIM: Color = Color::Rgb(98, 114, 164);  // #6272A4 (Dracula Comment)
pub const TEXT_SUBTLE: Color = Color::Rgb(166, 173, 200); // #A6ADC8 (Subtle text)

// ── Accent colors ───────────────────────────────────────────────────
pub const CYAN: Color = Color::Rgb(139, 233, 253);     // #8BE9FD (Dracula Cyan)
pub const ACCENT: Color = CYAN;                          // Alias: primary accent color
pub const PURPLE: Color = Color::Rgb(189, 147, 249);   // #BD93F9 (Dracula Purple)
pub const GREEN: Color = Color::Rgb(80, 250, 123);     // #50FA7B (Dracula Green)
pub const YELLOW: Color = Color::Rgb(241, 250, 140);   // #F1FA8C (Dracula Yellow)
pub const RED: Color = Color::Rgb(255, 85, 85);        // #FF5555 (Dracula Red)
pub const ORANGE: Color = Color::Rgb(255, 184, 108);   // #FFB86C (Dracula Orange)
pub const PINK: Color = Color::Rgb(255, 121, 198);     // #FF79C6 (Dracula Pink)

// ── UI chrome ───────────────────────────────────────────────────────
pub const BORDER: Color = Color::Rgb(68, 71, 90);      // #44475A (Dracula CurrentLine)
pub const SELECTED_BG: Color = Color::Rgb(68, 71, 90); // #44475A (Dracula CurrentLine)
pub const HEADER_BG: Color = Color::Rgb(33, 34, 44);   // #21222C (Darker header)
pub const FOOTER_BG: Color = Color::Rgb(33, 34, 44);   // #21222C (Darker footer)

// ── Status colors ───────────────────────────────────────────────────
pub const STATUS_RUNNING: Color = Color::Rgb(80, 250, 123);    // Green
pub const STATUS_STOPPED: Color = Color::Rgb(255, 85, 85);     // Red
pub const STATUS_CREATED: Color = Color::Rgb(241, 250, 140);   // Yellow
pub const STATUS_PAUSED: Color = Color::Rgb(139, 233, 253);    // Cyan
pub const STATUS_DEAD: Color = Color::Rgb(98, 114, 164);       // Comment (dim)

// ── Gauge colors ────────────────────────────────────────────────────
pub const GAUGE_LOW: Color = Color::Rgb(80, 250, 123);         // Green (< 50%)
pub const GAUGE_MED: Color = Color::Rgb(241, 250, 140);        // Yellow (50-80%)
pub const GAUGE_HIGH: Color = Color::Rgb(255, 85, 85);         // Red (> 80%)

// ── Tab titles for the header bar ───────────────────────────────────
pub const TAB_TITLES: &[&str] = &[
    "Containers",
    "Images",
    "Networks",
    "Volumes",
    "Stats",
    "Logs",
    "Extensions",
];

/// Approximate width per tab for click detection
pub const TAB_WIDTH: usize = 13;

// ── Style helpers ───────────────────────────────────────────────────
// Pre-built styles for common UI patterns to ensure consistency.

/// Default body text style
pub fn text_style() -> Style {
    Style::default().fg(TEXT)
}

/// Dimmed/comment text style
pub fn dim_style() -> Style {
    Style::default().fg(TEXT_DIM)
}

/// Subtle text style (between normal and dim)
pub fn subtle_style() -> Style {
    Style::default().fg(TEXT_SUBTLE)
}

/// Bold accent text style (for headings and highlights)
pub fn accent_style() -> Style {
    Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
}

/// Purple accent style (for secondary highlights)
pub fn purple_style() -> Style {
    Style::default().fg(PURPLE).add_modifier(Modifier::BOLD)
}

/// Selected item style (background highlight)
pub fn selected_style() -> Style {
    Style::default().bg(SELECTED_BG).fg(TEXT)
}

/// Selected item with bold accent
pub fn selected_accent_style() -> Style {
    Style::default().bg(SELECTED_BG).fg(CYAN).add_modifier(Modifier::BOLD)
}

/// Header bar style
pub fn header_style() -> Style {
    Style::default().bg(HEADER_BG).fg(TEXT)
}

/// Footer bar style
pub fn footer_style() -> Style {
    Style::default().bg(FOOTER_BG).fg(TEXT_DIM)
}

/// Status style based on container status name
pub fn status_style(status: &str) -> Style {
    match status {
        "Running" => Style::default().fg(STATUS_RUNNING).add_modifier(Modifier::BOLD),
        "Stopped" | "Dead" => Style::default().fg(STATUS_STOPPED),
        "Created" => Style::default().fg(STATUS_CREATED),
        "Paused" => Style::default().fg(STATUS_PAUSED),
        _ => Style::default().fg(TEXT_DIM),
    }
}

/// Gauge color based on percentage
pub fn gauge_color(percent: f64) -> Color {
    if percent < 50.0 {
        GAUGE_LOW
    } else if percent < 80.0 {
        GAUGE_MED
    } else {
        GAUGE_HIGH
    }
}

/// Border style for the main content area
pub fn border_style() -> Style {
    Style::default().fg(BORDER)
}

/// Tab active style
pub fn tab_active_style() -> Style {
    Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
}

/// Tab inactive style
pub fn tab_inactive_style() -> Style {
    Style::default().fg(TEXT_DIM)
}

/// Help popup style
pub fn help_style() -> Style {
    Style::default().fg(PURPLE).add_modifier(Modifier::BOLD)
}

/// Error/danger style
pub fn error_style() -> Style {
    Style::default().fg(RED).add_modifier(Modifier::BOLD)
}

/// Success style
pub fn success_style() -> Style {
    Style::default().fg(GREEN).add_modifier(Modifier::BOLD)
}

/// Warning style
pub fn warning_style() -> Style {
    Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)
}

/// Info style
pub fn info_style() -> Style {
    Style::default().fg(CYAN)
}

/// Key binding hint style (for keyboard shortcuts in the footer)
pub fn key_style() -> Style {
    Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)
}

/// Block title style
pub fn block_title_style() -> Style {
    Style::default().fg(PURPLE).add_modifier(Modifier::BOLD)
}

/// Table header style
pub fn table_header_style() -> Style {
    Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
}
