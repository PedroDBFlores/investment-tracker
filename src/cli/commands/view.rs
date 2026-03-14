use crate::core::Storage;
use crate::error::Result;
use crate::utils::display::{fmt_amount, fmt_return, load_currency_symbol};

pub fn run(id: String) -> Result<()> {
    let sym = load_currency_symbol();
    let storage = Storage::open();
    match storage.get_investment(&id)? {
        Some(inv) => {
            println!("Investment Details:");
            println!("------------------");
            println!("ID: {}", inv.id);
            println!("Type: {}", inv.investment_type);
            println!("Name: {}", inv.name);
            println!("Amount: {}", fmt_amount(&sym, inv.amount));
            println!("Date: {}", inv.date);
            if let Some(symbol) = &inv.symbol {
                println!("Symbol: {}", symbol);
            }
            if let Some(cv) = inv.current_value {
                println!("Current Value: {}", fmt_amount(&sym, cv));
                if let Some(roi) = inv.return_on_investment() {
                    println!(
                        "Return: {}",
                        fmt_return(&sym, roi, inv.return_percentage().unwrap_or(0.0))
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
                    "Total Dividends: {} ({} payments)",
                    fmt_amount(&sym, inv.total_dividends()),
                    inv.dividends.len()
                );
                if let Some(total_ret) = inv.total_return_with_dividends() {
                    println!(
                        "Total Return (incl. dividends): {} ({:.2}%)",
                        fmt_amount(&sym, total_ret),
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
