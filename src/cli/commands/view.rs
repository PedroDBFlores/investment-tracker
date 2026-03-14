use crate::core::Storage;
use crate::error::Result;

pub fn run(id: String) -> Result<()> {
    let storage = Storage::open();
    match storage.get_investment(&id)? {
        Some(inv) => {
            println!("Investment Details:");
            println!("------------------");
            println!("ID: {}", inv.id);
            println!("Type: {}", inv.investment_type);
            println!("Name: {}", inv.name);
            println!("Amount: ${:.2}", inv.amount);
            println!("Date: {}", inv.date);
            if let Some(symbol) = &inv.symbol {
                println!("Symbol: {}", symbol);
            }
            if let Some(cv) = inv.current_value {
                println!("Current Value: ${:.2}", cv);
                if let Some(roi) = inv.return_on_investment() {
                    println!(
                        "Return: ${:.2} ({:.2}%)",
                        roi,
                        inv.return_percentage().unwrap_or(0.0)
                    );
                }
            }
            if let Some(notes) = &inv.notes {
                println!("Notes: {}", notes);
            }
            if let Some(y) = inv.dividend_yield {
                println!("Dividend Yield: {:.2}%", y);
            }
            if let Some(f) = &inv.dividend_frequency {
                println!("Dividend Frequency: {}", f);
            }
            if !inv.dividends.is_empty() {
                println!(
                    "Total Dividends: ${:.2} ({} payments)",
                    inv.total_dividends(),
                    inv.dividends.len()
                );
                if let Some(total_ret) = inv.total_return_with_dividends() {
                    println!(
                        "Total Return (incl. dividends): ${:.2} ({:.2}%)",
                        total_ret,
                        inv.total_return_percentage_with_dividends().unwrap_or(0.0)
                    );
                }
            }
            println!("Created: {}", inv.created_at);
            println!("Updated: {}", inv.updated_at);
        }
        None => println!("Investment with ID '{}' not found.", id),
    }
    Ok(())
}
