use crate::core::{Investment, InvestmentType, Storage};
use crate::error::Result;
use crate::utils::display::spinner;

pub fn run(
    investment_type: String,
    name: String,
    amount: f64,
    date: Option<String>,
    symbol: Option<String>,
    dividend_yield: Option<f64>,
    dividend_frequency: Option<String>,
) -> Result<()> {
    let inv_type: InvestmentType = investment_type.parse().unwrap();
    let date = date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());

    let investment = Investment::new(
        String::new(),
        inv_type,
        name,
        symbol,
        amount,
        date,
        None,
        None,
        dividend_yield,
        dividend_frequency,
    )?;

    let pb = spinner("Saving investment…");
    let storage = Storage::open();
    let saved = storage.add_investment(investment)?;
    pb.finish_and_clear();

    println!("✓ Added investment: {} ({})", saved.name, saved.id);
    println!("  Type: {}", saved.investment_type);
    println!("  Amount: ${:.2}", saved.amount);
    println!("  Date: {}", saved.date);
    if let Some(sym) = &saved.symbol {
        println!("  Symbol: {}", sym);
    }
    Ok(())
}
