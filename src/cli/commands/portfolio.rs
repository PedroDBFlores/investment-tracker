use crate::core::{PortfolioAnalytics, Storage};
use crate::error::Result;
use crate::utils::display::{colors_enabled, fmt_amount, load_currency_symbol};
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

pub fn run() -> Result<()> {
    let sym = load_currency_symbol();
    let colors = colors_enabled();
    let storage = Storage::open();
    let analytics = PortfolioAnalytics::new(storage);
    let summary = analytics.get_summary()?;

    println!("📊 Portfolio Summary");
    println!("===================");

    let header_color = if colors { Color::Green } else { Color::White };
    let label_color = if colors { Color::Cyan } else { Color::White };
    let value_color = if colors { Color::Yellow } else { Color::White };

    let mut table = Table::new();
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Metric")
                .add_attribute(Attribute::Bold)
                .fg(header_color),
            Cell::new("Value")
                .add_attribute(Attribute::Bold)
                .fg(header_color),
        ]);

    table.add_row(vec![
        Cell::new("Total Investments").fg(label_color),
        Cell::new(summary.total_investments.to_string()).fg(value_color),
    ]);
    table.add_row(vec![
        Cell::new("Total Invested").fg(label_color),
        Cell::new(fmt_amount(&sym, summary.total_invested)).fg(value_color),
    ]);
    table.add_row(vec![
        Cell::new("Current Value").fg(label_color),
        Cell::new(fmt_amount(&sym, summary.total_current_value)).fg(value_color),
    ]);
    table.add_row(vec![
        Cell::new("Total Dividends").fg(label_color),
        Cell::new(fmt_amount(&sym, summary.total_dividends)).fg(value_color),
    ]);

    let roi_color = if colors {
        if summary.total_roi >= 0.0 {
            Color::Green
        } else {
            Color::Red
        }
    } else {
        Color::White
    };
    let roi_sign = if summary.total_roi >= 0.0 { "+" } else { "" };
    table.add_row(vec![
        Cell::new("Total ROI").fg(label_color),
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
    let alloc_header_color = if colors { Color::Blue } else { Color::White };
    alloc_table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Type")
                .add_attribute(Attribute::Bold)
                .fg(alloc_header_color),
            Cell::new("Count")
                .add_attribute(Attribute::Bold)
                .fg(alloc_header_color),
            Cell::new("Value")
                .add_attribute(Attribute::Bold)
                .fg(alloc_header_color),
            Cell::new("% of Portfolio")
                .add_attribute(Attribute::Bold)
                .fg(alloc_header_color),
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
        let type_color = if colors { Color::Green } else { Color::White };
        let pct_color = if colors {
            if percentage >= 20.0 {
                Color::Yellow
            } else {
                Color::Cyan
            }
        } else {
            Color::White
        };
        alloc_table.add_row(vec![
            Cell::new(type_name).fg(type_color),
            Cell::new(count.to_string()).fg(Color::White),
            Cell::new(fmt_amount(&sym, *value)).fg(value_color),
            Cell::new(format!("{:.1}%", percentage)).fg(pct_color),
        ]);
    }
    println!("{alloc_table}");

    Ok(())
}
