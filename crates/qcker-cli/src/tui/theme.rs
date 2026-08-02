use ratatui::style::Color;

// ── Dracula Theme ──────────────────────────────────────────────
// Official Dracula color palette adapted for terminal UI usage.
// See: https://draculatheme.com/contribute

// Background
pub const BG: Color = Color::Rgb(28, 30, 38);          // #1E1E2E (Catppuccin Mocha base)
pub const SURFACE: Color = Color::Rgb(40, 42, 54);     // #282A36 (Dracula Background)

// Foreground / Text
pub const TEXT: Color = Color::Rgb(248, 248, 242);     // #F8F8F2 (Dracula Foreground)
pub const TEXT_DIM: Color = Color::Rgb(98, 114, 164);  // #6272A4 (Dracula Comment)

// Accent colors
pub const ACCENT: Color = Color::Rgb(139, 233, 253);   // #8BE9FD (Dracula Cyan)
pub const PURPLE: Color = Color::Rgb(189, 147, 249);   // #BD93F9 (Dracula Purple)
#[allow(dead_code)]
pub const GREEN: Color = Color::Rgb(80, 250, 123);     // #50FA7B (Dracula Green)
pub const YELLOW: Color = Color::Rgb(241, 250, 140);   // #F1FA8C (Dracula Yellow)
pub const RED: Color = Color::Rgb(255, 85, 85);        // #FF5555 (Dracula Red)
#[allow(dead_code)]
pub const ORANGE: Color = Color::Rgb(255, 184, 108);   // #FFB86C (Dracula Orange)
pub const PINK: Color = Color::Rgb(255, 121, 198);     // #FF79C6 (Dracula Pink)

// UI chrome
pub const BORDER: Color = Color::Rgb(68, 71, 90);      // #44475A (Dracula CurrentLine)
pub const SELECTED_BG: Color = Color::Rgb(68, 71, 90); // #44475A (Dracula CurrentLine)
pub const HEADER_BG: Color = Color::Rgb(33, 34, 44);   // #21222C (Darker header)
pub const FOOTER_BG: Color = Color::Rgb(33, 34, 44);   // #21222C (Darker footer)

// Status colors
pub const STATUS_RUNNING: Color = Color::Rgb(80, 250, 123);    // Green
pub const STATUS_STOPPED: Color = Color::Rgb(255, 85, 85);     // Red
pub const STATUS_CREATED: Color = Color::Rgb(241, 250, 140);   // Yellow
pub const STATUS_PAUSED: Color = Color::Rgb(139, 233, 253);    // Cyan

// Gauge colors
pub const GAUGE_LOW: Color = Color::Rgb(80, 250, 123);         // Green (< 50%)
pub const GAUGE_MED: Color = Color::Rgb(241, 250, 140);        // Yellow (50-80%)
pub const GAUGE_HIGH: Color = Color::Rgb(255, 85, 85);         // Red (> 80%)

/// Tab titles for the header bar
#[allow(dead_code)]
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
#[allow(dead_code)]
pub const TAB_WIDTH: usize = 13;
