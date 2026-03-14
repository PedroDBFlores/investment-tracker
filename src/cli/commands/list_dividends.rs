use crate::core::Storage;
use crate::error::Result;
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

            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Date")
                        .add_attribute(Attribute::Bold)
                        .fg(Color::Cyan),
                    Cell::new("Amount")
                        .add_attribute(Attribute::Bold)
                        .fg(Color::Cyan),
                    Cell::new("Notes")
                        .add_attribute(Attribute::Bold)
                        .fg(Color::Cyan),
                ]);

            for entry in &inv.dividends {
                table.add_row(vec![
                    Cell::new(&entry.date).fg(Color::White),
                    Cell::new(format!("${:.2}", entry.amount)).fg(Color::Green),
                    Cell::new(entry.notes.as_deref().unwrap_or("")).fg(Color::White),
                ]);
            }

            // Total row
            table.add_row(vec![
                Cell::new("TOTAL")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Yellow),
                Cell::new(format!("${:.2}", inv.total_dividends()))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Yellow),
                Cell::new("").fg(Color::White),
            ]);

            println!("{table}");
            println!("  {} dividend payment(s) recorded.", inv.dividends.len());
        }
        None => println!("Investment with ID '{}' not found.", id),
    }
    Ok(())
}
