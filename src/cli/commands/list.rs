use crate::core::Storage;
use crate::error::Result;
use crate::utils::display::{
    colors_enabled, fmt_amount, fmt_return, load_currency_symbol, sparkline,
};
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

pub fn run() -> Result<()> {
    let storage = Storage::open();
    let investments = storage.get_all_investments()?;

    if investments.is_empty() {
        println!("No investments found.");
        return Ok(());
    }

    let sym = load_currency_symbol();
    let colors = colors_enabled();

    let header_color = if colors { Color::Cyan } else { Color::White };

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("ID")
                .add_attribute(Attribute::Bold)
                .fg(header_color),
            Cell::new("Name")
                .add_attribute(Attribute::Bold)
                .fg(header_color),
            Cell::new("Type")
                .add_attribute(Attribute::Bold)
                .fg(header_color),
            Cell::new("Invested")
                .add_attribute(Attribute::Bold)
                .fg(header_color),
            Cell::new("Current Value")
                .add_attribute(Attribute::Bold)
                .fg(header_color),
            Cell::new("Return")
                .add_attribute(Attribute::Bold)
                .fg(header_color),
            Cell::new("Units")
                .add_attribute(Attribute::Bold)
                .fg(header_color),
            Cell::new("Trend")
                .add_attribute(Attribute::Bold)
                .fg(header_color),
            Cell::new("Created")
                .add_attribute(Attribute::Bold)
                .fg(header_color),
            Cell::new("Date")
                .add_attribute(Attribute::Bold)
                .fg(header_color),
        ]);

    for inv in &investments {
        // Shorten UUID to first 8 chars — copy-pasteable as a prefix ID
        let short_id = if inv.id.len() > 8 {
            inv.id[..8].to_string()
        } else {
            inv.id.clone()
        };

        let current_value_cell = match inv.current_value {
            Some(cv) => {
                let color = if colors { Color::Yellow } else { Color::White };
                Cell::new(fmt_amount(&sym, cv)).fg(color)
            }
            None => {
                let color = if colors {
                    Color::DarkGrey
                } else {
                    Color::White
                };
                Cell::new("—").fg(color)
            }
        };

        let return_cell = match (inv.return_on_investment(), inv.return_percentage()) {
            (Some(roi), Some(pct)) => {
                let color = if colors {
                    if roi >= 0.0 { Color::Green } else { Color::Red }
                } else {
                    Color::White
                };
                Cell::new(fmt_return(&sym, roi, pct)).fg(color)
            }
            _ => {
                let color = if colors {
                    Color::DarkGrey
                } else {
                    Color::White
                };
                Cell::new("—").fg(color)
            }
        };

        let units_cell = match inv.units {
            Some(u) => Cell::new(format!("{}", u)).fg(Color::White),
            None => {
                let color = if colors {
                    Color::DarkGrey
                } else {
                    Color::White
                };
                Cell::new("—").fg(color)
            }
        };

        let trend_cell = {
            let sorted = inv.sorted_price_history();
            if sorted.len() >= 2 {
                let prices: Vec<f64> = sorted.iter().map(|e| e.price).collect();
                let spark = sparkline(&prices);
                Cell::new(spark).fg(Color::White)
            } else {
                let color = if colors {
                    Color::DarkGrey
                } else {
                    Color::White
                };
                Cell::new("—").fg(color)
            }
        };

        let id_color = if colors {
            Color::DarkGrey
        } else {
            Color::White
        };

        table.add_row(vec![
            Cell::new(short_id).fg(id_color),
            Cell::new(&inv.name).fg(Color::White),
            Cell::new(inv.investment_type.to_string()).fg(Color::White),
            Cell::new(fmt_amount(&sym, inv.amount)).fg(Color::White),
            current_value_cell,
            return_cell,
            units_cell,
            trend_cell,
            // Created date is a full timestamp like "YYYY-MM-DD HH:MM:SS" — display only the date portion
            {
                let created_date = inv
                    .created_at
                    .split_whitespace()
                    .next()
                    .unwrap_or(&inv.created_at)
                    .to_string();
                Cell::new(created_date).fg(Color::White)
            },
            Cell::new(&inv.date).fg(Color::White),
        ]);
    }

    println!("{table}");
    println!("Total: {} investment(s)", investments.len());

    Ok(())
}
