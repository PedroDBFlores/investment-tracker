use crate::core::Storage;
use crate::error::{InvestmentError, Result};
use crate::utils::display::spinner;

pub fn run(
    id: String,
    amount: Option<f64>,
    current_value: Option<f64>,
    date: Option<String>,
    notes: Option<String>,
) -> Result<()> {
    let storage = Storage::open();
    match storage.get_investment(&id)? {
        Some(mut inv) => {
            let mut updated = false;
            if let Some(a) = amount {
                inv.update_amount(a)?;
                updated = true;
            }
            if let Some(cv) = current_value {
                inv.update_current_value(cv)?;
                updated = true;
            }
            if let Some(d) = date {
                inv.date = d;
                updated = true;
            }
            if let Some(n) = notes {
                inv.notes = Some(n);
                updated = true;
            }
            if updated {
                let pb = spinner("Saving changes…");
                storage.update_investment(&inv)?;
                pb.finish_and_clear();
                println!("✓ Updated investment: {}", inv.id);
                if let Some(ref n) = inv.notes {
                    println!("  Notes: {}", n);
                }
            } else {
                println!("No changes made to investment: {}", inv.id);
            }
        }
        None => {
            return Err(InvestmentError::NotFound(format!(
                "Investment with ID '{}' not found",
                id
            ))
            .into());
        }
    }
    Ok(())
}
