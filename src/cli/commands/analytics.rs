use crate::core::Storage;
use crate::error::Result;
use crate::utils::display::{fmt_amount, load_currency_symbol};
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

pub fn run() -> Result<()> {
    let sym = load_currency_symbol();
    let storage = Storage::open();
    let investments = storage.get_all_investments()?;

    if investments.is_empty() {
        println!("No investments found.");
        return Ok(());
    }

    println!("📊 Investment Analytics");
    println!("{}", "═".repeat(50));

    // ── Collect investments that have a current_value ─────────────────────────
    let mut with_value: Vec<_> = investments
        .iter()
        .filter(|i| i.current_value.is_some())
        .collect();

    // Sort by return % descending
    with_value.sort_by(|a, b| {
        let pa = a.return_percentage().unwrap_or(f64::NEG_INFINITY);
        let pb = b.return_percentage().unwrap_or(f64::NEG_INFINITY);
        pb.partial_cmp(&pa).unwrap_or(std::cmp::Ordering::Equal)
    });

    // ── Best performers ───────────────────────────────────────────────────────
    println!("\n🏆 Best Performers (Top 3 by Return %)");
    if with_value.is_empty() {
        println!("  No investments with current value data.");
    } else {
        let mut best_table = Table::new();
        best_table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Name")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new("Type")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new("Invested")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new("Current Value")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new("Return %")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
            ]);

        for inv in with_value.iter().take(3) {
            let pct = inv.return_percentage().unwrap_or(0.0);
            let cv = inv.current_value.unwrap_or(0.0);
            let color = if pct >= 0.0 { Color::Green } else { Color::Red };
            let sign = if pct >= 0.0 { "+" } else { "" };
            best_table.add_row(vec![
                Cell::new(&inv.name).fg(Color::White),
                Cell::new(inv.investment_type.to_string()).fg(Color::White),
                Cell::new(fmt_amount(&sym, inv.amount)).fg(Color::White),
                Cell::new(fmt_amount(&sym, cv)).fg(Color::Yellow),
                Cell::new(format!("{}{:.2}%", sign, pct))
                    .fg(color)
                    .add_attribute(Attribute::Bold),
            ]);
        }
        println!("{best_table}");
    }

    // ── Worst performers ──────────────────────────────────────────────────────
    println!("\n📉 Worst Performers (Bottom 3 by Return %)");
    if with_value.is_empty() {
        println!("  No investments with current value data.");
    } else {
        let mut worst_table = Table::new();
        worst_table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Name")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new("Type")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new("Invested")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new("Current Value")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new("Return %")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
            ]);

        // Worst = tail of the already-sorted-descending list
        let skip = if with_value.len() > 3 {
            with_value.len() - 3
        } else {
            0
        };
        for inv in with_value.iter().skip(skip) {
            let pct = inv.return_percentage().unwrap_or(0.0);
            let cv = inv.current_value.unwrap_or(0.0);
            let color = if pct >= 0.0 { Color::Green } else { Color::Red };
            let sign = if pct >= 0.0 { "+" } else { "" };
            worst_table.add_row(vec![
                Cell::new(&inv.name).fg(Color::White),
                Cell::new(inv.investment_type.to_string()).fg(Color::White),
                Cell::new(fmt_amount(&sym, inv.amount)).fg(Color::White),
                Cell::new(fmt_amount(&sym, cv)).fg(Color::Yellow),
                Cell::new(format!("{}{:.2}%", sign, pct))
                    .fg(color)
                    .add_attribute(Attribute::Bold),
            ]);
        }
        println!("{worst_table}");
    }

    // ── Highest dividend earners ──────────────────────────────────────────────
    let mut dividend_earners: Vec<_> = investments
        .iter()
        .filter(|i| i.total_dividends() > 0.0)
        .collect();

    println!("\n💰 Highest Dividend Earners (Top 3)");
    if dividend_earners.is_empty() {
        println!("  No dividend payments recorded yet.");
    } else {
        dividend_earners.sort_by(|a, b| {
            b.total_dividends()
                .partial_cmp(&a.total_dividends())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut div_table = Table::new();
        div_table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Name")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new("Type")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new("Payments")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new("Total Dividends")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
            ]);

        for inv in dividend_earners.iter().take(3) {
            div_table.add_row(vec![
                Cell::new(&inv.name).fg(Color::White),
                Cell::new(inv.investment_type.to_string()).fg(Color::White),
                Cell::new(inv.dividends.len().to_string()).fg(Color::White),
                Cell::new(fmt_amount(&sym, inv.total_dividends()))
                    .fg(Color::Green)
                    .add_attribute(Attribute::Bold),
            ]);
        }
        println!("{div_table}");
    }

    // ── Return statistics ─────────────────────────────────────────────────────
    println!("\n📈 Return Statistics");
    if with_value.is_empty() {
        println!("  No investments with current value data.");
    } else {
        let percentages: Vec<f64> = with_value
            .iter()
            .filter_map(|i| i.return_percentage())
            .collect();

        let n = percentages.len() as f64;
        let mean = percentages.iter().sum::<f64>() / n;
        let variance = percentages.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();

        let min_pct = percentages.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_pct = percentages
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);

        let mut stats_table = Table::new();
        stats_table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Metric")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new("Value")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
            ]);

        stats_table.add_row(vec![
            Cell::new("Investments analysed").fg(Color::White),
            Cell::new(with_value.len().to_string()).fg(Color::Yellow),
        ]);

        let mean_color = if mean >= 0.0 {
            Color::Green
        } else {
            Color::Red
        };
        let mean_sign = if mean >= 0.0 { "+" } else { "" };
        stats_table.add_row(vec![
            Cell::new("Mean Return %").fg(Color::White),
            Cell::new(format!("{}{:.2}%", mean_sign, mean)).fg(mean_color),
        ]);

        stats_table.add_row(vec![
            Cell::new("Std Dev of Return %").fg(Color::White),
            Cell::new(format!("{:.2}%", std_dev)).fg(Color::Yellow),
        ]);

        let min_color = if min_pct >= 0.0 {
            Color::Green
        } else {
            Color::Red
        };
        let min_sign = if min_pct >= 0.0 { "+" } else { "" };
        stats_table.add_row(vec![
            Cell::new("Min Return %").fg(Color::White),
            Cell::new(format!("{}{:.2}%", min_sign, min_pct)).fg(min_color),
        ]);

        let max_color = if max_pct >= 0.0 {
            Color::Green
        } else {
            Color::Red
        };
        let max_sign = if max_pct >= 0.0 { "+" } else { "" };
        stats_table.add_row(vec![
            Cell::new("Max Return %").fg(Color::White),
            Cell::new(format!("{}{:.2}%", max_sign, max_pct)).fg(max_color),
        ]);

        println!("{stats_table}");
    }

    Ok(())
}
