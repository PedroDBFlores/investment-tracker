use crate::core::models::{DividendEntry, PriceEntry};
use crate::core::{Investment, InvestmentType, Storage};
use crate::error::Result;
use crate::utils::display::spinner;
use serde::Deserialize;
use std::collections::HashMap;

// ── Typed CSV row structs ─────────────────────────────────────────────────────
// Using proper Deserialize structs instead of serde_json::Value ensures that
// csv::Reader correctly maps header columns to named fields.

#[derive(Debug, Deserialize)]
struct InvestmentRow {
    id: String,
    #[serde(rename = "type")]
    investment_type: String,
    name: String,
    #[serde(default)]
    symbol: String,
    amount: f64,
    date: String,
    #[serde(default)]
    current_value: Option<f64>,
    #[serde(default)]
    notes: String,
    #[serde(default)]
    dividend_yield: Option<f64>,
    #[serde(default)]
    dividend_frequency: String,
    #[serde(default)]
    created_at: String,
    #[serde(default)]
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct PriceHistoryRow {
    investment_id: String,
    date: String,
    price: f64,
    #[serde(default)]
    notes: String,
}

#[derive(Debug, Deserialize)]
struct DividendRow {
    investment_id: String,
    date: String,
    amount: f64,
    #[serde(default)]
    notes: String,
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run(path: String) -> Result<()> {
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let pb = spinner(format!("Reading {}…", path).as_str());

    let mut investments: Vec<Investment> = match ext.as_str() {
        "csv" => {
            let mut rdr = csv::Reader::from_path(&path)?;
            let mut imported = Vec::new();

            for result in rdr.deserialize() {
                let row: InvestmentRow = result?;

                let inv_type: InvestmentType = row.investment_type.parse().unwrap();

                // Build the investment via Investment::new to run all validations,
                // then immediately restore the original timestamps so a CSV
                // round-trip does not silently overwrite created_at/updated_at (#6).
                let mut investment = Investment::new(
                    row.id.clone(),
                    inv_type,
                    row.name,
                    if row.symbol.is_empty() {
                        None
                    } else {
                        Some(row.symbol)
                    },
                    row.amount,
                    row.date,
                    row.current_value,
                    if row.notes.is_empty() {
                        None
                    } else {
                        Some(row.notes)
                    },
                    row.dividend_yield,
                    if row.dividend_frequency.is_empty() {
                        None
                    } else {
                        Some(row.dividend_frequency)
                    },
                )?;

                // Restore original timestamps if the CSV had them.
                if !row.created_at.is_empty() {
                    investment.created_at = row.created_at;
                }
                if !row.updated_at.is_empty() {
                    investment.updated_at = row.updated_at;
                }

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

    // ── Sidecar CSV files for price history and dividends (#2) ────────────────
    // When the main file is a CSV, look for companion files next to it:
    //   <stem>_price_history.csv  and  <stem>_dividends.csv
    // These are produced by the CSV exporter and restore the sub-records that
    // cannot be represented in the flat main CSV.
    if ext == "csv" {
        let base = std::path::Path::new(&path);
        let stem = base
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("portfolio");
        let parent = base.parent().unwrap_or(std::path::Path::new("."));

        let mut price_map: HashMap<String, Vec<PriceEntry>> = HashMap::new();
        let mut dividend_map: HashMap<String, Vec<DividendEntry>> = HashMap::new();

        let price_path = parent.join(format!("{}_price_history.csv", stem));
        if price_path.exists() {
            pb.set_message(format!(
                "Reading price history from {}…",
                price_path.display()
            ));
            let mut rdr = csv::Reader::from_path(&price_path)?;
            for result in rdr.deserialize() {
                let row: PriceHistoryRow = result?;
                let entry = PriceEntry {
                    date: row.date,
                    price: row.price,
                    notes: if row.notes.is_empty() {
                        None
                    } else {
                        Some(row.notes)
                    },
                };
                price_map.entry(row.investment_id).or_default().push(entry);
            }
        }

        let dividend_path = parent.join(format!("{}_dividends.csv", stem));
        if dividend_path.exists() {
            pb.set_message(format!(
                "Reading dividends from {}…",
                dividend_path.display()
            ));
            let mut rdr = csv::Reader::from_path(&dividend_path)?;
            for result in rdr.deserialize() {
                let row: DividendRow = result?;
                let entry = DividendEntry {
                    date: row.date,
                    amount: row.amount,
                    notes: if row.notes.is_empty() {
                        None
                    } else {
                        Some(row.notes)
                    },
                };
                dividend_map
                    .entry(row.investment_id)
                    .or_default()
                    .push(entry);
            }
        }

        // Attach the sub-records to each investment.
        for inv in &mut investments {
            if let Some(entries) = price_map.remove(&inv.id) {
                inv.price_history = entries;
                inv.price_history.sort_by(|a, b| a.date.cmp(&b.date));
            }
            if let Some(entries) = dividend_map.remove(&inv.id) {
                inv.dividends = entries;
                inv.dividends.sort_by(|a, b| a.date.cmp(&b.date));
            }
        }
    }

    // ── Duplicate detection ───────────────────────────────────────────────────
    pb.set_message(format!(
        "Checking {} records for duplicates…",
        investments.len()
    ));

    let storage = Storage::open();
    let existing = storage.get_all_investments()?;
    let existing_ids: std::collections::HashSet<String> =
        existing.iter().map(|i| i.id.clone()).collect();

    let duplicates: Vec<String> = investments
        .iter()
        .filter(|i| existing_ids.contains(&i.id))
        .map(|i| i.id.clone())
        .collect();

    let new_investments: Vec<Investment> = investments
        .into_iter()
        .filter(|i| !existing_ids.contains(&i.id))
        .collect();

    let count = new_investments.len();

    // ── Single bulk save instead of N individual writes (#3) ─────────────────
    if !new_investments.is_empty() {
        pb.set_message(format!("Saving {} new investments…", count));
        storage.add_investments(new_investments)?;
    }

    pb.finish_and_clear();

    if !duplicates.is_empty() {
        println!(
            "⚠️  Skipped {} investment(s) with duplicate IDs: {}",
            duplicates.len(),
            duplicates.join(", ")
        );
    }

    println!("✓ Imported {} new investment(s) from {}", count, path);

    Ok(())
}
