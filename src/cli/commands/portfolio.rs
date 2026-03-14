use crate::core::{PortfolioAnalytics, Storage};
use crate::error::Result;
use crate::utils::display::{fmt_amount, load_currency_symbol};
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

pub fn run() -> Result<()> {
    let sym = load_currency_symbol();
    let storage = Storage::open();
    let analytics = PortfolioAnalytics::new(storage);
    let summary = analytics.get_summary()?;

    println!("📊 Portfolio Summary");
    println!("===================");

    let mut table = Table::new();
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Metric")
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
            Cell::new("Value")
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
        ]);

    table.add_row(vec![
        Cell::new("Total Investments").fg(Color::Cyan),
        Cell::new(summary.total_investments.to_string()).fg(Color::Yellow),
    ]);
    table.add_row(vec![
        Cell::new("Total Invested").fg(Color::Cyan),
        Cell::new(fmt_amount(&sym, summary.total_invested)).fg(Color::Yellow),
    ]);
    table.add_row(vec![
        Cell::new("Current Value").fg(Color::Cyan),
        Cell::new(fmt_amount(&sym, summary.total_current_value)).fg(Color::Yellow),
    ]);
    table.add_row(vec![
        Cell::new("Total Dividends").fg(Color::Cyan),
        Cell::new(fmt_amount(&sym, summary.total_dividends)).fg(Color::Yellow),
    ]);

    let roi_color = if summary.total_roi >= 0.0 {
        Color::Green
    } else {
        Color::Red
    };
    let roi_sign = if summary.total_roi >= 0.0 { "+" } else { "" };
    table.add_row(vec![
        Cell::new("Total ROI").fg(Color::Cyan),
        Cell::new(format!(
            "{}{} ({}{:.2}%)",
            roi_sign,
            fmt_amount(&sym, summary.total_roi),
            roi_sign,
            summary.total_roi_percentage
        ))
        .fg(roi_color)
        .add_attribute(Attribute::Bold),
    ]);
    println!("{table}");

    println!("\n📈 Allocation by Type:");
    let mut alloc_table = Table::new();
    alloc_table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Type")
                .add_attribute(Attribute::Bold)
                .fg(Color::Blue),
            Cell::new("Count")
                .add_attribute(Attribute::Bold)
                .fg(Color::Blue),
            Cell::new("Value")
                .add_attribute(Attribute::Bold)
                .fg(Color::Blue),
            Cell::new("% of Portfolio")
                .add_attribute(Attribute::Bold)
                .fg(Color::Blue),
        ])
        .load_preset(UTF8_FULL);

    let mut types: Vec<_> = summary.allocation_by_type.iter().collect();
    types.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (type_name, value) in types {
        let count = summary.count_by_type.get(type_name).copied().unwrap_or(0);
        let percentage = if summary.total_current_value > 0.0 {
            (value / summary.total_current_value) * 100.0
        } else {
            0.0
        };
        let pct_color = if percentage >= 20.0 {
            Color::Yellow
        } else {
            Color::Cyan
        };
        alloc_table.add_row(vec![
            Cell::new(type_name).fg(Color::Green),
            Cell::new(count.to_string()).fg(Color::White),
            Cell::new(fmt_amount(&sym, *value)).fg(Color::Yellow),
            Cell::new(format!("{:.1}%", percentage)).fg(pct_color),
        ]);
    }
    println!("{alloc_table}");

    Ok(())
}
