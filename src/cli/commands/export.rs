use crate::core::Storage;
use crate::error::Result;
use crate::utils::display::spinner;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct InvestmentRow<'a> {
    id: &'a str,
    #[serde(rename = "type")]
    investment_type: String,
    name: &'a str,
    symbol: &'a str,
    amount: f64,
    date: &'a str,
    current_value: Option<f64>,
    notes: &'a str,
    dividend_yield: Option<f64>,
    dividend_frequency: &'a str,
    created_at: &'a str,
    updated_at: &'a str,
}

#[derive(Serialize)]
struct PriceHistoryRow<'a> {
    investment_id: &'a str,
    date: &'a str,
    price: f64,
    notes: &'a str,
}

#[derive(Serialize)]
struct DividendRow<'a> {
    investment_id: &'a str,
    date: &'a str,
    amount: f64,
    notes: &'a str,
}

pub fn run(path: String, format: String) -> Result<()> {
    let pb = spinner("Loading investments…");
    let storage = Storage::open();
    let investments = storage.get_all_investments()?;
    pb.set_message(format!("Exporting {} investments…", investments.len()));

    match format.to_lowercase().as_str() {
        "csv" => {
            // ── Main investments file ─────────────────────────────────────────
            let mut wtr = csv::Writer::from_path(&path)?;
            for inv in &investments {
                wtr.serialize(InvestmentRow {
                    id: &inv.id,
                    investment_type: format!("{}", inv.investment_type),
                    name: &inv.name,
                    symbol: inv.symbol.as_deref().unwrap_or(""),
                    amount: inv.amount,
                    date: &inv.date,
                    current_value: inv.current_value,
                    notes: inv.notes.as_deref().unwrap_or(""),
                    dividend_yield: inv.dividend_yield,
                    dividend_frequency: inv.dividend_frequency.as_deref().unwrap_or(""),
                    created_at: &inv.created_at,
                    updated_at: &inv.updated_at,
                })?;
            }
            wtr.flush()?;

            // ── Derive sidecar paths from the main file's stem ────────────────
            let base = Path::new(&path);
            let stem = base
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("portfolio");
            let parent = base.parent().unwrap_or(Path::new("."));

            // ── Price history sidecar ─────────────────────────────────────────
            let price_path = parent.join(format!("{}_price_history.csv", stem));
            let mut price_wtr = csv::Writer::from_path(&price_path)?;
            let mut price_count = 0usize;
            for inv in &investments {
                for entry in &inv.price_history {
                    price_wtr.serialize(PriceHistoryRow {
                        investment_id: &inv.id,
                        date: &entry.date,
                        price: entry.price,
                        notes: entry.notes.as_deref().unwrap_or(""),
                    })?;
                    price_count += 1;
                }
            }
            price_wtr.flush()?;

            // ── Dividends sidecar ─────────────────────────────────────────────
            let dividend_path = parent.join(format!("{}_dividends.csv", stem));
            let mut div_wtr = csv::Writer::from_path(&dividend_path)?;
            let mut dividend_count = 0usize;
            for inv in &investments {
                for entry in &inv.dividends {
                    div_wtr.serialize(DividendRow {
                        investment_id: &inv.id,
                        date: &entry.date,
                        amount: entry.amount,
                        notes: entry.notes.as_deref().unwrap_or(""),
                    })?;
                    dividend_count += 1;
                }
            }
            div_wtr.flush()?;

            pb.finish_and_clear();
            println!("✓ Exported {} investment(s) to {}", investments.len(), path);
            println!(
                "  📈 Price history: {} record(s) → {}",
                price_count,
                price_path.display()
            );
            println!(
                "  💰 Dividends:     {} record(s) → {}",
                dividend_count,
                dividend_path.display()
            );
            if price_count > 0 || dividend_count > 0 {
                println!("  ℹ️  Import all three files together to restore the full portfolio.");
            }
        }
        "json" => {
            let json_data = serde_json::to_string_pretty(&investments)?;
            std::fs::write(&path, json_data)?;
            pb.finish_and_clear();
            println!(
                "✓ Exported {} investment(s) to {} (JSON)",
                investments.len(),
                path
            );
        }
        other => {
            pb.finish_and_clear();
            return Err(anyhow::anyhow!(
                "Unsupported export format: '{}'. Use 'csv' or 'json'",
                other
            ));
        }
    }

    Ok(())
}
