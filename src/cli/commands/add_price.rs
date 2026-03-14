use crate::core::Storage;
use crate::error::{InvestmentError, Result};
use crate::utils::display::{fmt_amount, load_currency_symbol, spinner};

pub fn run(
    id: String,
    price: f64,
    date: Option<String>,
    notes: Option<String>,
    unit_price: Option<f64>,
) -> Result<()> {
    let date = date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let pb = spinner("Recording price entry…");
    let storage = Storage::open();
    match storage.mutate_investment(&id, |inv| {
        inv.add_price_entry(date.clone(), price, notes.clone(), unit_price)
    })? {
        Some(inv) => {
            pb.finish_and_clear();
            let cur = load_currency_symbol();
            println!(
                "✓ Recorded price {} for {} on {}",
                fmt_amount(&cur, price),
                inv.name,
                date
            );
            if let Some(up) = unit_price {
                println!("  Unit Price: {}", fmt_amount(&cur, up));
            }
            println!("  Price history: {} entries", inv.price_history.len());
            if let Some(twr) = inv.time_weighted_return() {
                println!("  Time-weighted return: {:.2}%", twr);
            }
        }
        None => {
            pb.finish_and_clear();
            return Err(InvestmentError::NotFound(format!(
                "Investment with ID '{}' not found",
                id
            ))
            .into());
        }
    }
    Ok(())
}
