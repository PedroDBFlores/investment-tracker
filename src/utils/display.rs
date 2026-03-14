use chrono::Utc;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn now_timestamp() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

#[allow(dead_code)]
pub fn format_currency(value: f64) -> String {
    format!("${:.2}", value)
}

#[allow(dead_code)]
pub fn format_return(roi: f64, pct: f64) -> String {
    format!("${:.2} ({:.2}%)", roi, pct)
}

/// Creates a spinner with a consistent style and the given message.
/// Call `.finish_and_clear()` or `.finish_with_message(...)` when done.
pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}
