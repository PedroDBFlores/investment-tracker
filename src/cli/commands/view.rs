use crate::core::Storage;
use crate::error::Result;
use crate::utils::display::{fmt_amount, fmt_return, load_currency_symbol, sparkline};

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

            // Units / cost basis
            if let Some(units) = inv.units {
                println!("Units Held: {}", units);
            }
            if let Some(cbpu) = inv.cost_basis_per_unit() {
                println!("Cost Basis/Unit: {}", fmt_amount(&sym, cbpu));
            }
            if let Some(remaining) = inv.remaining_units() {
                if inv
                    .units
                    .map(|u| (u - remaining).abs() > 1e-9)
                    .unwrap_or(false)
                {
                    println!("Remaining Units: {}", remaining);
                }
            }

            if let Some(cv) = inv.current_value {
                println!("Current Value: {}", fmt_amount(&sym, cv));

                // Current price per unit: prefer the unit_price on the latest price entry,
                // fall back to current_value / units if units are tracked.
                let latest_unit_price = inv
                    .sorted_price_history()
                    .last()
                    .and_then(|e| e.unit_price)
                    .or_else(|| inv.units.map(|u| if u > 0.0 { cv / u } else { 0.0 }));
                if let Some(up) = latest_unit_price {
                    println!("Current Price/Unit: {}", fmt_amount(&sym, up));
                }

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

            // Sparkline
            let sorted_history = inv.sorted_price_history();
            if sorted_history.len() >= 2 {
                let prices: Vec<f64> = sorted_history.iter().map(|e| e.price).collect();
                let spark = sparkline(&prices);
                println!(
                    "Price Trend:  {}  ({} entries)",
                    spark,
                    sorted_history.len()
                );
            }

            // Sales history
            if !inv.sales.is_empty() {
                println!();
                println!("Sales History ({} sale(s)):", inv.sales.len());
                println!(
                    "  {:<12}  {:>10}  {:>12}  {:>12}  {:>14}  Notes",
                    "Date", "Units Sold", "Price/Unit", "Proceeds", "Realized Gain"
                );
                println!("  {}", "─".repeat(80));
                for sale in &inv.sales {
                    let gain_str = if sale.realized_gain >= 0.0 {
                        format!("+{}", fmt_amount(&sym, sale.realized_gain))
                    } else {
                        format!("-{}", fmt_amount(&sym, sale.realized_gain.abs()))
                    };
                    println!(
                        "  {:<12}  {:>10.4}  {:>12}  {:>12}  {:>14}  {}",
                        sale.date,
                        sale.units_sold,
                        fmt_amount(&sym, sale.sale_price_per_unit),
                        fmt_amount(&sym, sale.total_proceeds),
                        gain_str,
                        sale.notes.as_deref().unwrap_or(""),
                    );
                }
                let total_gain = inv.total_realized_gain();
                let sign = if total_gain >= 0.0 { "+" } else { "" };
                println!(
                    "  Total Realized Gain: {}{}",
                    sign,
                    fmt_amount(&sym, total_gain)
                );
            }

            println!("Created: {}", inv.created_at);
            println!("Updated: {}", inv.updated_at);
        }
        None => println!("Investment with ID '{}' not found.", id),
    }
    Ok(())
}
