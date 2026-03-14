use crate::core::Storage;
use crate::error::{InvestmentError, Result};
use crate::utils::display::{colors_enabled, fmt_amount, load_currency_symbol};
use chrono::Local;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

/// Parse a range string into a cutoff date string (YYYY-MM-DD), or None for "all".
fn range_cutoff(range: &str) -> Option<String> {
    let days: i64 = match range {
        "1m" => 30,
        "3m" => 90,
        "6m" => 180,
        "1y" => 365,
        _ => return None,
    };
    let cutoff = Local::now()
        .date_naive()
        .checked_sub_days(chrono::Days::new(days as u64))?;
    Some(cutoff.format("%Y-%m-%d").to_string())
}

pub fn run(id: Option<String>, range: String) -> Result<()> {
    let sym = load_currency_symbol();
    let colors = colors_enabled();
    let storage = Storage::open();
    let cutoff = range_cutoff(&range);

    match id {
        Some(inv_id) => {
            // ── Detailed view for a single investment ──────────────────────────
            let inv = storage.get_investment(&inv_id)?.ok_or_else(|| {
                InvestmentError::NotFound(format!("Investment with ID '{}' not found", inv_id))
            })?;

            println!("📈 Performance: {}", inv.name);
            println!("{}", "─".repeat(50));
            println!("  ID:           {}", inv.id);
            println!("  Type:         {}", inv.investment_type);
            if let Some(ticker) = &inv.symbol {
                println!("  Symbol:       {}", ticker);
            }
            println!("  Invested:     {}", fmt_amount(&sym, inv.amount));

            match inv.current_value {
                Some(cv) => {
                    let roi = cv - inv.amount;
                    let pct = (roi / inv.amount) * 100.0;
                    let sign = if roi >= 0.0 { "+" } else { "" };
                    println!("  Current Value: {}", fmt_amount(&sym, cv));
                    println!(
                        "  Return:        {}{} ({}{:.2}%)",
                        sign,
                        fmt_amount(&sym, roi.abs()),
                        sign,
                        pct.abs()
                    );
                }
                None => {
                    println!("  Current Value: N/A");
                    println!("  Return:        N/A");
                }
            }

            // Filter history by range for TWR display
            let filtered: Vec<_> = {
                let mut entries: Vec<_> = inv.price_history.iter().collect();
                entries.sort_by(|a, b| a.date.cmp(&b.date));
                if let Some(ref cut) = cutoff {
                    entries.retain(|e| e.date.as_str() >= cut.as_str());
                }
                entries
            };

            let twr_display = if filtered.len() >= 2 {
                let earliest = filtered.first().unwrap().price;
                let latest = filtered.last().unwrap().price;
                if earliest > 0.0 {
                    let twr = (latest - earliest) / earliest * 100.0;
                    let sign = if twr >= 0.0 { "+" } else { "" };
                    format!("{}{:.2}%  (range: {})", sign, twr, range)
                } else {
                    "N/A".to_string()
                }
            } else if let Some(pct) = inv.return_percentage() {
                let sign = if pct >= 0.0 { "+" } else { "" };
                format!("{}{:.2}%  (from cost basis)", sign, pct)
            } else {
                "N/A".to_string()
            };
            println!("  TWR:           {}", twr_display);

            // ── Price history table ────────────────────────────────────────────
            if inv.price_history.is_empty() {
                println!("\n  No price history recorded.");
            } else {
                println!(
                    "\n  Price History ({} total entries):",
                    inv.price_history.len()
                );

                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec![
                        Cell::new("Date")
                            .add_attribute(Attribute::Bold)
                            .fg(if colors { Color::Cyan } else { Color::White }),
                        Cell::new("Price")
                            .add_attribute(Attribute::Bold)
                            .fg(if colors { Color::Cyan } else { Color::White }),
                        Cell::new("Change")
                            .add_attribute(Attribute::Bold)
                            .fg(if colors { Color::Cyan } else { Color::White }),
                        Cell::new("Notes")
                            .add_attribute(Attribute::Bold)
                            .fg(if colors { Color::Cyan } else { Color::White }),
                    ]);

                let sorted = inv.sorted_price_history();
                let mut prev_price: Option<f64> = None;

                for entry in &sorted {
                    let change_cell = match prev_price {
                        Some(prev) if prev > 0.0 => {
                            let delta = entry.price - prev;
                            let pct = delta / prev * 100.0;
                            let sign = if delta >= 0.0 { "+" } else { "" };
                            let color = if colors {
                                if delta >= 0.0 {
                                    Color::Green
                                } else {
                                    Color::Red
                                }
                            } else {
                                Color::White
                            };
                            Cell::new(format!("{}{:.2} ({}{:.2}%)", sign, delta, sign, pct))
                                .fg(color)
                        }
                        _ => Cell::new("—").fg(if colors {
                            Color::DarkGrey
                        } else {
                            Color::White
                        }),
                    };

                    let in_range = cutoff
                        .as_ref()
                        .map(|cut| entry.date.as_str() >= cut.as_str())
                        .unwrap_or(true);

                    let date_cell = if !colors || in_range {
                        Cell::new(&entry.date).fg(Color::White)
                    } else {
                        Cell::new(&entry.date).fg(Color::DarkGrey)
                    };

                    table.add_row(vec![
                        date_cell,
                        Cell::new(fmt_amount(&sym, entry.price)).fg(if colors {
                            Color::Yellow
                        } else {
                            Color::White
                        }),
                        change_cell,
                        Cell::new(entry.notes.as_deref().unwrap_or("")).fg(Color::White),
                    ]);

                    prev_price = Some(entry.price);
                }

                println!("{table}");
            }
        }

        None => {
            // ── Summary table for all investments ─────────────────────────────
            let investments = storage.get_all_investments()?;

            if investments.is_empty() {
                println!("No investments found.");
                return Ok(());
            }

            println!("📊 Performance Report");
            if range != "all" {
                println!("   Range: {}", range);
            }
            println!();

            // Sort: investments with current_value first (by return % desc),
            // then those without current_value.
            let mut with_value: Vec<_> = investments
                .iter()
                .filter(|i| i.current_value.is_some())
                .collect();
            let without_value: Vec<_> = investments
                .iter()
                .filter(|i| i.current_value.is_none())
                .collect();

            with_value.sort_by(|a, b| {
                let pct_a = a.return_percentage().unwrap_or(f64::NEG_INFINITY);
                let pct_b = b.return_percentage().unwrap_or(f64::NEG_INFINITY);
                pct_b
                    .partial_cmp(&pct_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Name")
                        .add_attribute(Attribute::Bold)
                        .fg(if colors { Color::Cyan } else { Color::White }),
                    Cell::new("Type")
                        .add_attribute(Attribute::Bold)
                        .fg(if colors { Color::Cyan } else { Color::White }),
                    Cell::new("Invested")
                        .add_attribute(Attribute::Bold)
                        .fg(if colors { Color::Cyan } else { Color::White }),
                    Cell::new("Current Value")
                        .add_attribute(Attribute::Bold)
                        .fg(if colors { Color::Cyan } else { Color::White }),
                    Cell::new("Return ($)")
                        .add_attribute(Attribute::Bold)
                        .fg(if colors { Color::Cyan } else { Color::White }),
                    Cell::new("Return (%)")
                        .add_attribute(Attribute::Bold)
                        .fg(if colors { Color::Cyan } else { Color::White }),
                    Cell::new("TWR (%)")
                        .add_attribute(Attribute::Bold)
                        .fg(if colors { Color::Cyan } else { Color::White }),
                ]);

            let all_rows: Vec<_> = with_value.iter().chain(without_value.iter()).collect();

            for inv in all_rows {
                let invested_cell = Cell::new(fmt_amount(&sym, inv.amount)).fg(Color::White);
                let type_cell = Cell::new(inv.investment_type.to_string()).fg(Color::White);
                let name_cell = Cell::new(&inv.name).fg(Color::White);

                let (cv_cell, roi_cell, pct_cell) = match inv.current_value {
                    Some(cv) => {
                        let roi = cv - inv.amount;
                        let pct = roi / inv.amount * 100.0;
                        let color = if colors {
                            if roi >= 0.0 { Color::Green } else { Color::Red }
                        } else {
                            Color::White
                        };
                        let sign = if roi >= 0.0 { "+" } else { "" };
                        (
                            Cell::new(fmt_amount(&sym, cv)).fg(if colors {
                                Color::Yellow
                            } else {
                                Color::White
                            }),
                            Cell::new(format!("{}{}", sign, fmt_amount(&sym, roi.abs()))).fg(color),
                            Cell::new(format!("{}{:.2}%", sign, pct)).fg(color),
                        )
                    }
                    None => (
                        Cell::new("N/A").fg(if colors {
                            Color::DarkGrey
                        } else {
                            Color::White
                        }),
                        Cell::new("N/A").fg(if colors {
                            Color::DarkGrey
                        } else {
                            Color::White
                        }),
                        Cell::new("N/A").fg(if colors {
                            Color::DarkGrey
                        } else {
                            Color::White
                        }),
                    ),
                };

                // Compute TWR filtered to the requested range
                let filtered_entries: Vec<_> = {
                    let mut entries: Vec<_> = inv.price_history.iter().collect();
                    entries.sort_by(|a, b| a.date.cmp(&b.date));
                    if let Some(ref cut) = cutoff {
                        entries.retain(|e| e.date.as_str() >= cut.as_str());
                    }
                    entries
                };

                let twr_cell = if filtered_entries.len() >= 2 {
                    let earliest = filtered_entries.first().unwrap().price;
                    let latest = filtered_entries.last().unwrap().price;
                    if earliest > 0.0 {
                        let twr = (latest - earliest) / earliest * 100.0;
                        let color = if colors {
                            if twr >= 0.0 { Color::Green } else { Color::Red }
                        } else {
                            Color::White
                        };
                        let sign = if twr >= 0.0 { "+" } else { "" };
                        Cell::new(format!("{}{:.2}%", sign, twr)).fg(color)
                    } else {
                        Cell::new("N/A").fg(if colors {
                            Color::DarkGrey
                        } else {
                            Color::White
                        })
                    }
                } else if let Some(pct) = inv.return_percentage() {
                    let color = if colors {
                        if pct >= 0.0 { Color::Green } else { Color::Red }
                    } else {
                        Color::White
                    };
                    let sign = if pct >= 0.0 { "+" } else { "" };
                    Cell::new(format!("{}{:.2}%*", sign, pct)).fg(color)
                } else {
                    Cell::new("N/A").fg(if colors {
                        Color::DarkGrey
                    } else {
                        Color::White
                    })
                };

                table.add_row(vec![
                    name_cell,
                    type_cell,
                    invested_cell,
                    cv_cell,
                    roi_cell,
                    pct_cell,
                    twr_cell,
                ]);
            }

            println!("{table}");
            println!("  * TWR falls back to cost-basis return when price history is unavailable.");
        }
    }

    Ok(())
}
