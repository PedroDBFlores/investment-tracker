use crate::core::Storage;
use crate::error::Result;
use crate::utils::display::{colors_enabled, fmt_amount, load_currency_symbol};
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

pub fn run(id: String) -> Result<()> {
    let storage = Storage::open();
    match storage.get_investment(&id)? {
        Some(inv) => {
            println!("💰 Dividend History: {}", inv.name);
            println!("{}", "─".repeat(50));

            if inv.dividends.is_empty() {
                println!("No dividends recorded for this investment.");
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
                    Cell::new("Date")
                        .add_attribute(Attribute::Bold)
                        .fg(header_color),
                    Cell::new("Amount")
                        .add_attribute(Attribute::Bold)
                        .fg(header_color),
                    Cell::new("Notes")
                        .add_attribute(Attribute::Bold)
                        .fg(header_color),
                ]);

            let amount_color = if colors { Color::Green } else { Color::White };
            for entry in &inv.dividends {
                table.add_row(vec![
                    Cell::new(&entry.date).fg(Color::White),
                    Cell::new(fmt_amount(&sym, entry.amount)).fg(amount_color),
                    Cell::new(entry.notes.as_deref().unwrap_or("")).fg(Color::White),
                ]);
            }

            // Total row
            let total_color = if colors { Color::Yellow } else { Color::White };
            table.add_row(vec![
                Cell::new("TOTAL")
                    .add_attribute(Attribute::Bold)
                    .fg(total_color),
                Cell::new(fmt_amount(&sym, inv.total_dividends()))
                    .add_attribute(Attribute::Bold)
                    .fg(total_color),
                Cell::new("").fg(Color::White),
            ]);

            println!("{table}");
            println!("  {} dividend payment(s) recorded.", inv.dividends.len());
        }
        None => println!("Investment with ID '{}' not found.", id),
    }
    Ok(())
}
