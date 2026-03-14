use crate::core::Storage;
use crate::error::{InvestmentError, Result};
use crate::utils::display::spinner;

pub fn run(id: String) -> Result<()> {
    let pb = spinner("Deleting investment…");
    let storage = Storage::open();
    match storage.delete_investment(&id)? {
        Some(deleted) => {
            pb.finish_and_clear();
            println!("✓ Deleted investment: {} ({})", deleted.id, deleted.name);
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
