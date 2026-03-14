use crate::core::Storage;
use crate::error::{InvestmentError, Result};
use crate::utils::display::spinner;

pub fn run(id: String, amount: f64, date: Option<String>, notes: Option<String>) -> Result<()> {
    let date = date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let pb = spinner("Recording dividend…");
    let storage = Storage::open();
    match storage.mutate_investment(&id, |inv| {
        inv.add_dividend(date.clone(), amount, notes.clone())
    })? {
        Some(inv) => {
            pb.finish_and_clear();
            println!(
                "✓ Recorded dividend ${:.2} for {} on {}",
                amount, inv.name, date
            );
            println!("  Total dividends: ${:.2}", inv.total_dividends());
            println!("  Dividend entries: {}", inv.dividends.len());
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
