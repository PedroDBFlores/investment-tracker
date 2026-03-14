use crate::core::Storage;
use crate::error::{InvestmentError, Result};
use crate::utils::display::{fmt_amount, load_currency_symbol, spinner};

pub fn run(
    id: String,
    units_sold: f64,
    price_per_unit: f64,
    date: Option<String>,
    notes: Option<String>,
) -> Result<()> {
    let date = date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let pb = spinner("Recording sale…");
    let storage = Storage::open();

    let mut inv = match storage.get_investment(&id)? {
        Some(i) => i,
        None => {
            pb.finish_and_clear();
            return Err(InvestmentError::NotFound(format!(
                "Investment with ID '{}' not found",
                id
            ))
            .into());
        }
    };

    let entry = inv.sell(date.clone(), units_sold, price_per_unit, notes)?;

    storage.update_investment(&inv)?;
    pb.finish_and_clear();

    let cur = load_currency_symbol();
    println!("✓ Recorded sale for: {} ({})", inv.name, inv.id);
    println!("  Date:             {}", entry.date);
    println!("  Units Sold:       {}", entry.units_sold);
    println!(
        "  Price/Unit:       {}",
        fmt_amount(&cur, entry.sale_price_per_unit)
    );
    println!(
        "  Total Proceeds:   {}",
        fmt_amount(&cur, entry.total_proceeds)
    );

    let gain_str = if entry.realized_gain >= 0.0 {
        format!("+{}", fmt_amount(&cur, entry.realized_gain))
    } else {
        format!("-{}", fmt_amount(&cur, entry.realized_gain.abs()))
    };
    println!("  Realized Gain:    {}", gain_str);

    if let Some(remaining) = inv.remaining_units() {
        println!("  Remaining Units:  {}", remaining);
    }
    if let Some(cv) = inv.current_value {
        println!("  Remaining Value:  {}", fmt_amount(&cur, cv));
    }
    if let Some(total_gain) = {
        let tg = inv.total_realized_gain();
        if inv.sales.len() > 1 { Some(tg) } else { None }
    } {
        let sign = if total_gain >= 0.0 { "+" } else { "" };
        println!(
            "  Total Realized Gain (all sales): {}{}",
            sign,
            fmt_amount(&cur, total_gain)
        );
    }

    Ok(())
}
