use crate::core::Storage;
use crate::error::Result;
use crate::utils::display::{fmt_amount, fmt_return, load_currency_symbol};
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

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("ID")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Name")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Type")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Invested")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Current Value")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Return")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Date")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
        ]);

    for inv in &investments {
        // Shorten UUID to first 8 chars + ellipsis
        let short_id = if inv.id.len() > 8 {
            format!("{}…", &inv.id[..8])
        } else {
            inv.id.clone()
        };

        let current_value_cell = match inv.current_value {
            Some(cv) => Cell::new(fmt_amount(&sym, cv)).fg(Color::Yellow),
            None => Cell::new("—").fg(Color::DarkGrey),
        };

        let return_cell = match (inv.return_on_investment(), inv.return_percentage()) {
            (Some(roi), Some(pct)) => {
                let color = if roi >= 0.0 { Color::Green } else { Color::Red };
                Cell::new(fmt_return(&sym, roi, pct)).fg(color)
            }
            _ => Cell::new("—").fg(Color::DarkGrey),
        };

        table.add_row(vec![
            Cell::new(short_id).fg(Color::DarkGrey),
            Cell::new(&inv.name).fg(Color::White),
            Cell::new(inv.investment_type.to_string()).fg(Color::White),
            Cell::new(fmt_amount(&sym, inv.amount)).fg(Color::White),
            current_value_cell,
            return_cell,
            Cell::new(&inv.date).fg(Color::White),
        ]);
    }

    println!("{table}");
    println!("Total: {} investment(s)", investments.len());

    Ok(())
}
