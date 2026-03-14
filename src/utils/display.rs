use chrono::Utc;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Returns `true` when colour output is enabled (the default), or `false` when
/// the user has set `color-output = false` in their config.
pub fn colors_enabled() -> bool {
    crate::core::config::Config::load()
        .map(|c| c.color_output.unwrap_or(true))
        .unwrap_or(true)
}

pub fn now_timestamp() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Loads the currency symbol from config (e.g. "€" when currency is set to "EUR").
/// Falls back to "$" if config cannot be read.
pub fn load_currency_symbol() -> String {
    crate::core::config::Config::load()
        .map(|c| c.currency_symbol())
        .unwrap_or_else(|_| "$".to_string())
}

/// Formats a monetary value with the configured currency symbol.
/// e.g. `fmt_amount("€", 1234.5)` → `"€1234.50"`
pub fn fmt_amount(symbol: &str, value: f64) -> String {
    format!("{}{:.2}", symbol, value)
}

/// Formats a return as `"+€150.00 (+15.00%)"` using the given symbol.
pub fn fmt_return(symbol: &str, roi: f64, pct: f64) -> String {
    let sign = if roi >= 0.0 { "+" } else { "" };
    format!(
        "{}{}{:.2} ({}{:.2}%)",
        sign,
        symbol,
        roi.abs(),
        sign,
        pct.abs()
    )
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
