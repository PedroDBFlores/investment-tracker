use crate::core::Storage;
use crate::error::Result;
use crate::utils::display::spinner;

pub fn run(path: String, format: String) -> Result<()> {
    let pb = spinner("Loading investments…");
    let storage = Storage::open();
    let investments = storage.get_all_investments()?;
    pb.set_message(format!("Exporting {} investments…", investments.len()));

    match format.to_lowercase().as_str() {
        "csv" => {
            let mut wtr = csv::Writer::from_path(&path)?;
            for inv in &investments {
                wtr.serialize(serde_json::json!({
                    "id": inv.id,
                    "type": format!("{}", inv.investment_type),
                    "name": inv.name,
                    "symbol": inv.symbol,
                    "amount": inv.amount,
                    "date": inv.date,
                    "current_value": inv.current_value,
                    "notes": inv.notes,
                    "dividend_yield": inv.dividend_yield,
                    "dividend_frequency": inv.dividend_frequency,
                }))?;
            }
            wtr.flush()?;
        }
        "json" => {
            let json_data = serde_json::to_string_pretty(&investments)?;
            std::fs::write(&path, json_data)?;
        }
        other => {
            pb.finish_and_clear();
            return Err(anyhow::anyhow!(
                "Unsupported export format: {}. Use 'csv' or 'json'",
                other
            ));
        }
    }

    pb.finish_and_clear();
    println!(
        "✓ Exported {} investments to {} ({})",
        investments.len(),
        path,
        format.to_uppercase()
    );
    Ok(())
}
