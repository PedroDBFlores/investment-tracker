use crate::core::{Investment, InvestmentType, Storage};
use crate::error::Result;
use crate::utils::display::spinner;

pub fn run(path: String) -> Result<()> {
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let pb = spinner(format!("Reading {}…", path).as_str());

    let investments: Vec<Investment> = match ext.as_str() {
        "csv" => {
            let mut rdr = csv::Reader::from_path(&path)?;
            let mut imported = Vec::new();
            for result in rdr.deserialize() {
                let record: serde_json::Value = result?;
                let inv_type: InvestmentType =
                    record["type"].as_str().unwrap_or("other").parse().unwrap();
                let investment = Investment::new(
                    record["id"].as_str().unwrap_or("").to_string(),
                    inv_type,
                    record["name"].as_str().unwrap_or("").to_string(),
                    record["symbol"].as_str().map(|s| s.to_string()),
                    record["amount"].as_f64().unwrap_or(0.0),
                    record["date"].as_str().unwrap_or("").to_string(),
                    record["current_value"].as_f64(),
                    record["notes"].as_str().map(|s| s.to_string()),
                    record["dividend_yield"].as_f64(),
                    record["dividend_frequency"].as_str().map(|s| s.to_string()),
                )?;
                imported.push(investment);
            }
            imported
        }
        "json" => {
            let data = std::fs::read_to_string(&path)?;
            serde_json::from_str(&data)?
        }
        other => {
            pb.finish_and_clear();
            return Err(anyhow::anyhow!(
                "Unsupported import format: {}. Use CSV or JSON files",
                other
            ));
        }
    };

    pb.set_message(format!(
        "Checking {} records for duplicates…",
        investments.len()
    ));

    let storage = Storage::open();
    let existing = storage.get_all_investments()?;
    let existing_ids: std::collections::HashSet<_> = existing.iter().map(|i| &i.id).collect();

    let duplicates: Vec<_> = investments
        .iter()
        .filter(|i| existing_ids.contains(&i.id))
        .map(|i| i.id.clone())
        .collect();

    let new_investments: Vec<_> = investments
        .into_iter()
        .filter(|i| !existing_ids.contains(&i.id))
        .collect();

    pb.set_message(format!("Saving {} new investments…", new_investments.len()));

    for inv in &new_investments {
        storage.add_investment(inv.clone())?;
    }

    pb.finish_and_clear();

    if !duplicates.is_empty() {
        println!(
            "⚠️  Warning: {} investment(s) with duplicate IDs found: {}",
            duplicates.len(),
            duplicates.join(", ")
        );
        println!("   These investments were skipped to avoid overwriting existing data.");
    }

    println!(
        "✓ Imported {} new investments from {}",
        new_investments.len(),
        path
    );

    if !duplicates.is_empty() {
        println!("⚠️  Skipped {} duplicate investments", duplicates.len());
    }

    Ok(())
}
