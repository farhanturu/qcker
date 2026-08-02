use qcker_error::{QckerError, ErrorSeverity};

/// Display a QckerError in the specified format.
///
/// Formats:
/// - "json"  → structured JSON output
/// - "quiet" → code + message only
/// - other   → pretty-printed with color, hints, and source chain
#[allow(dead_code)]
pub fn display_error(err: &QckerError, format: &str) {
    match format {
        "json" => display_json(err),
        "quiet" => display_quiet(err),
        _ => display_pretty(err),
    }
}

fn display_pretty(err: &QckerError) {
    let (icon, color) = match err.severity() {
        ErrorSeverity::Info => ("[INFO]", "\x1b[34m"),
        ErrorSeverity::Warning => ("[WARN]", "\x1b[33m"),
        ErrorSeverity::Error => ("[ERR]", "\x1b[31m"),
        ErrorSeverity::Critical => ("[CRIT]", "\x1b[1;31m"),
    };

    let reset = "\x1b[0m";
    let dim = "\x1b[2m";

    eprintln!("{}{}{} {} {}", color, icon, reset, err.error_code(), err.message);

    if let Some(suggestion) = &err.suggestion {
        eprintln!("{}  Hint: {}{}", dim, suggestion, reset);
    }

    if let Some(source) = &err.source {
        eprintln!("{}  Caused by: {}{}", dim, source, reset);
    }

    if err.retryable() {
        eprintln!("{}  This operation can be retried.{}", dim, reset);
    }
}

fn display_json(err: &QckerError) {
    eprintln!("{}", err.to_json());
}

fn display_quiet(err: &QckerError) {
    eprintln!("{}: {}", err.error_code(), err.message);
}
