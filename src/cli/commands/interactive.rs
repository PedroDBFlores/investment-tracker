use crate::core::{Investment, InvestmentType, Storage};
use crate::error::Result;
use crate::utils::display::spinner;
use chrono::Local;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

// ── Top-level menu actions ────────────────────────────────────────────────────

enum MenuAction {
    AddInvestment,
    ListInvestments,
    UpdateInvestment,
    RecordPrice,
    RecordDividend,
    DeleteInvestment,
    Quit,
}

impl MenuAction {
    fn label(&self) -> &'static str {
        match self {
            MenuAction::AddInvestment => "➕  Add a new investment",
            MenuAction::ListInvestments => "📋  List investments",
            MenuAction::UpdateInvestment => "✏️   Update an investment",
            MenuAction::RecordPrice => "💹  Record a price entry",
            MenuAction::RecordDividend => "💰  Record a dividend payment",
            MenuAction::DeleteInvestment => "🗑️   Delete an investment",
            MenuAction::Quit => "🚪  Quit",
        }
    }

    fn all() -> Vec<MenuAction> {
        vec![
            MenuAction::AddInvestment,
            MenuAction::ListInvestments,
            MenuAction::UpdateInvestment,
            MenuAction::RecordPrice,
            MenuAction::RecordDividend,
            MenuAction::DeleteInvestment,
            MenuAction::Quit,
        ]
    }

    fn labels() -> Vec<&'static str> {
        Self::all().iter().map(|a| a.label()).collect()
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run() -> Result<()> {
    let theme = ColorfulTheme::default();

    println!();
    println!("  💼  Investment Tracker — Interactive Mode");
    println!("  {}", "─".repeat(42));
    println!();

    loop {
        let actions = MenuAction::all();
        let labels = MenuAction::labels();

        let selection = Select::with_theme(&theme)
            .with_prompt("What would you like to do?")
            .items(&labels)
            .default(0)
            .interact()?;

        println!();

        match actions[selection] {
            MenuAction::AddInvestment => interactive_add(&theme)?,
            MenuAction::ListInvestments => interactive_list()?,
            MenuAction::UpdateInvestment => interactive_update(&theme)?,
            MenuAction::RecordPrice => interactive_add_price(&theme)?,
            MenuAction::RecordDividend => interactive_add_dividend(&theme)?,
            MenuAction::DeleteInvestment => interactive_delete(&theme)?,
            MenuAction::Quit => {
                println!("  Goodbye! 👋");
                println!();
                break;
            }
        }

        println!();
    }

    Ok(())
}

// ── Add investment ────────────────────────────────────────────────────────────

fn interactive_add(theme: &ColorfulTheme) -> Result<()> {
    println!("  ➕  Add a New Investment");
    println!("  {}", "─".repeat(30));

    // Investment type
    let type_labels = vec![
        "Stock", "ETF", "Mutual Fund", "Deposit", "Bond", "Crypto", "Other",
    ];
    let type_idx = Select::with_theme(theme)
        .with_prompt("Investment type")
        .items(&type_labels)
        .default(0)
        .interact()?;

    let inv_type = match type_idx {
        0 => InvestmentType::Stock,
        1 => InvestmentType::ETF,
        2 => InvestmentType::MutualFund,
        3 => InvestmentType::Deposit,
        4 => InvestmentType::Bond,
        5 => InvestmentType::Crypto,
        _ => {
            let custom: String = Input::with_theme(theme)
                .with_prompt("Enter custom type name")
                .interact_text()?;
            InvestmentType::Other(custom)
        }
    };

    // Name
    let name: String = Input::with_theme(theme)
        .with_prompt("Investment name")
        .interact_text()?;

    // Symbol (optional)
    let symbol_str: String = Input::with_theme(theme)
        .with_prompt("Ticker symbol (leave blank to skip)")
        .allow_empty(true)
        .interact_text()?;
    let symbol = if symbol_str.trim().is_empty() {
        None
    } else {
        Some(symbol_str.trim().to_string())
    };

    // Amount
    let amount: f64 = loop {
        let raw: String = Input::with_theme(theme)
            .with_prompt("Amount invested (e.g. 1500.00)")
            .interact_text()?;
        match raw.trim().parse::<f64>() {
            Ok(v) if v > 0.0 => break v,
            Ok(_) => println!("  ⚠  Amount must be greater than zero."),
            Err(_) => println!("  ⚠  Please enter a valid number."),
        }
    };

    // Date
    let today = Local::now().format("%Y-%m-%d").to_string();
    let date: String = loop {
        let raw: String = Input::with_theme(theme)
            .with_prompt("Purchase date (YYYY-MM-DD)")
            .default(today.clone())
            .interact_text()?;
        let d = raw.trim().to_string();
        if d.len() == 10 && d.chars().nth(4) == Some('-') && d.chars().nth(7) == Some('-') {
            break d;
        }
        println!("  ⚠  Date must be in YYYY-MM-DD format.");
    };

    // Notes (optional)
    let notes_str: String = Input::with_theme(theme)
        .with_prompt("Notes (leave blank to skip)")
        .allow_empty(true)
        .interact_text()?;
    let notes = if notes_str.trim().is_empty() {
        None
    } else {
        Some(notes_str.trim().to_string())
    };

    // Dividend yield (optional)
    let div_yield_str: String = Input::with_theme(theme)
        .with_prompt("Dividend yield % (leave blank to skip)")
        .allow_empty(true)
        .interact_text()?;
    let dividend_yield = if div_yield_str.trim().is_empty() {
        None
    } else {
        div_yield_str.trim().parse::<f64>().ok()
    };

    // Dividend frequency (optional, only if yield was given)
    let dividend_frequency = if dividend_yield.is_some() {
        let freq_str: String = Input::with_theme(theme)
            .with_prompt("Dividend frequency (e.g. monthly, quarterly, annual — blank to skip)")
            .allow_empty(true)
            .interact_text()?;
        if freq_str.trim().is_empty() {
            None
        } else {
            Some(freq_str.trim().to_string())
        }
    } else {
        None
    };

    // Confirm
    println!();
    println!("  Review:");
    println!("    Type:   {}", inv_type);
    println!("    Name:   {}", name);
    if let Some(ref s) = symbol {
        println!("    Symbol: {}", s);
    }
    println!("    Amount: ${:.2}", amount);
    println!("    Date:   {}", date);
    if let Some(ref n) = notes {
        println!("    Notes:  {}", n);
    }
    println!();

    if !Confirm::with_theme(theme)
        .with_prompt("Save this investment?")
        .default(true)
        .interact()?
    {
        println!("  Cancelled.");
        return Ok(());
    }

    let investment = Investment::new(
        String::new(),
        inv_type,
        name,
        symbol,
        amount,
        date,
        None,
        notes,
        dividend_yield,
        dividend_frequency,
    )?;

    let pb = spinner("Saving investment…");
    let storage = Storage::open();
    let saved = storage.add_investment(investment)?;
    pb.finish_and_clear();

    println!("  ✓ Added investment: {} ({})", saved.name, saved.id);

    Ok(())
}

// ── List investments ──────────────────────────────────────────────────────────

fn interactive_list() -> Result<()> {
    let pb = spinner("Loading investments…");
    let storage = Storage::open();
    let investments = storage.get_all_investments()?;
    pb.finish_and_clear();

    if investments.is_empty() {
        println!("  No investments found.");
        return Ok(());
    }

    println!("  📋  Your Investments  ({} total)", investments.len());
    println!("  {}", "─".repeat(60));

    for (i, inv) in investments.iter().enumerate() {
        let short_id = &inv.id[..8.min(inv.id.len())];
        let value_str = match inv.current_value {
            Some(cv) => {
                let roi = cv - inv.amount;
                let sign = if roi >= 0.0 { "+" } else { "" };
                let pct = roi / inv.amount * 100.0;
                format!("${:.2}  ({}{:.1}%)", cv, sign, pct)
            }
            None => "—".to_string(),
        };
        println!(
            "  {:>2}.  [{}…]  {}  |  {}  |  invested ${:.2}  |  current {}",
            i + 1,
            short_id,
            inv.name,
            inv.investment_type,
            inv.amount,
            value_str,
        );
    }

    Ok(())
}

// ── Pick an investment by interactive list ────────────────────────────────────

fn pick_investment(theme: &ColorfulTheme, prompt: &str) -> Result<Option<String>> {
    let pb = spinner("Loading investments…");
    let storage = Storage::open();
    let investments = storage.get_all_investments()?;
    pb.finish_and_clear();

    if investments.is_empty() {
        println!("  No investments found.");
        return Ok(None);
    }

    let labels: Vec<String> = investments
        .iter()
        .map(|inv| {
            let short_id = &inv.id[..8.min(inv.id.len())];
            let value_str = inv
                .current_value
                .map(|cv| format!("  current ${:.2}", cv))
                .unwrap_or_default();
            format!(
                "[{}…]  {}  (invested ${:.2}{})",
                short_id, inv.name, inv.amount, value_str
            )
        })
        .collect();

    let idx = Select::with_theme(theme)
        .with_prompt(prompt)
        .items(&labels)
        .default(0)
        .interact()?;

    Ok(Some(investments[idx].id.clone()))
}

// ── Update investment ─────────────────────────────────────────────────────────

fn interactive_update(theme: &ColorfulTheme) -> Result<()> {
    println!("  ✏️   Update an Investment");
    println!("  {}", "─".repeat(30));

    let id = match pick_investment(theme, "Select investment to update")? {
        Some(id) => id,
        None => return Ok(()),
    };

    let pb_load = spinner("Loading investment…");
    let storage = Storage::open();
    let mut inv = match storage.get_investment(&id)? {
        Some(i) => i,
        None => {
            pb_load.finish_and_clear();
            println!("  Investment not found.");
            return Ok(());
        }
    };
    pb_load.finish_and_clear();

    println!();
    println!("  Updating: {} ({})", inv.name, inv.investment_type);
    println!("  Leave a field blank to keep its current value.");
    println!();

    // Amount
    let amount_str: String = Input::with_theme(theme)
        .with_prompt(format!("New amount invested (current: ${:.2})", inv.amount))
        .allow_empty(true)
        .interact_text()?;
    if !amount_str.trim().is_empty() {
        match amount_str.trim().parse::<f64>() {
            Ok(v) if v > 0.0 => inv.update_amount(v)?,
            _ => println!("  ⚠  Invalid amount — skipped."),
        }
    }

    // Current value
    let cv_str: String = Input::with_theme(theme)
        .with_prompt(format!(
            "New current value (current: {})",
            inv.current_value
                .map(|v| format!("${:.2}", v))
                .unwrap_or_else(|| "—".to_string())
        ))
        .allow_empty(true)
        .interact_text()?;
    if !cv_str.trim().is_empty() {
        match cv_str.trim().parse::<f64>() {
            Ok(v) if v >= 0.0 => inv.update_current_value(v)?,
            _ => println!("  ⚠  Invalid value — skipped."),
        }
    }

    // Date
    let date_str: String = Input::with_theme(theme)
        .with_prompt(format!("New purchase date (current: {})", inv.date))
        .allow_empty(true)
        .interact_text()?;
    if !date_str.trim().is_empty() {
        let d = date_str.trim().to_string();
        if d.len() == 10 && d.chars().nth(4) == Some('-') && d.chars().nth(7) == Some('-') {
            inv.date = d;
        } else {
            println!("  ⚠  Invalid date format — skipped.");
        }
    }

    println!();
    if !Confirm::with_theme(theme)
        .with_prompt("Save changes?")
        .default(true)
        .interact()?
    {
        println!("  Cancelled.");
        return Ok(());
    }

    let pb = spinner("Saving changes…");
    storage.update_investment(&inv)?;
    pb.finish_and_clear();
    println!("  ✓ Updated investment: {}", inv.id);

    Ok(())
}

// ── Record price entry ────────────────────────────────────────────────────────

fn interactive_add_price(theme: &ColorfulTheme) -> Result<()> {
    println!("  💹  Record a Price Entry");
    println!("  {}", "─".repeat(30));

    let id = match pick_investment(theme, "Select investment")? {
        Some(id) => id,
        None => return Ok(()),
    };

    let price: f64 = loop {
        let raw: String = Input::with_theme(theme)
            .with_prompt("Price / current value")
            .interact_text()?;
        match raw.trim().parse::<f64>() {
            Ok(v) if v > 0.0 => break v,
            Ok(_) => println!("  ⚠  Price must be greater than zero."),
            Err(_) => println!("  ⚠  Please enter a valid number."),
        }
    };

    let today = Local::now().format("%Y-%m-%d").to_string();
    let date: String = loop {
        let raw: String = Input::with_theme(theme)
            .with_prompt("Date of price entry (YYYY-MM-DD)")
            .default(today.clone())
            .interact_text()?;
        let d = raw.trim().to_string();
        if d.len() == 10 && d.chars().nth(4) == Some('-') && d.chars().nth(7) == Some('-') {
            break d;
        }
        println!("  ⚠  Date must be in YYYY-MM-DD format.");
    };

    let notes_str: String = Input::with_theme(theme)
        .with_prompt("Notes (leave blank to skip)")
        .allow_empty(true)
        .interact_text()?;
    let notes = if notes_str.trim().is_empty() {
        None
    } else {
        Some(notes_str.trim().to_string())
    };

    let pb = spinner("Saving price entry…");
    let storage = Storage::open();
    match storage.mutate_investment(&id, |inv| {
        inv.add_price_entry(date.clone(), price, notes.clone())
    })? {
        Some(inv) => {
            pb.finish_and_clear();
            println!("  ✓ Recorded price ${:.2} for {} on {}", price, inv.name, date);
            println!("    Price history: {} entries", inv.price_history.len());
            if let Some(twr) = inv.time_weighted_return() {
                println!("    Time-weighted return: {:.2}%", twr);
            }
        }
        None => {
            pb.finish_and_clear();
            println!("  Investment not found.");
        }
    }

    Ok(())
}

// ── Record dividend ───────────────────────────────────────────────────────────

fn interactive_add_dividend(theme: &ColorfulTheme) -> Result<()> {
    println!("  💰  Record a Dividend Payment");
    println!("  {}", "─".repeat(30));

    let id = match pick_investment(theme, "Select investment")? {
        Some(id) => id,
        None => return Ok(()),
    };

    let amount: f64 = loop {
        let raw: String = Input::with_theme(theme)
            .with_prompt("Dividend amount received")
            .interact_text()?;
        match raw.trim().parse::<f64>() {
            Ok(v) if v > 0.0 => break v,
            Ok(_) => println!("  ⚠  Amount must be greater than zero."),
            Err(_) => println!("  ⚠  Please enter a valid number."),
        }
    };

    let today = Local::now().format("%Y-%m-%d").to_string();
    let date: String = loop {
        let raw: String = Input::with_theme(theme)
            .with_prompt("Date received (YYYY-MM-DD)")
            .default(today.clone())
            .interact_text()?;
        let d = raw.trim().to_string();
        if d.len() == 10 && d.chars().nth(4) == Some('-') && d.chars().nth(7) == Some('-') {
            break d;
        }
        println!("  ⚠  Date must be in YYYY-MM-DD format.");
    };

    let notes_str: String = Input::with_theme(theme)
        .with_prompt("Notes (leave blank to skip)")
        .allow_empty(true)
        .interact_text()?;
    let notes = if notes_str.trim().is_empty() {
        None
    } else {
        Some(notes_str.trim().to_string())
    };

    let pb = spinner("Saving dividend…");
    let storage = Storage::open();
    match storage.mutate_investment(&id, |inv| {
        inv.add_dividend(date.clone(), amount, notes.clone())
    })? {
        Some(inv) => {
            pb.finish_and_clear();
            println!("  ✓ Recorded dividend ${:.2} for {} on {}", amount, inv.name, date);
            println!("    Total dividends: ${:.2}", inv.total_dividends());
        }
        None => {
            pb.finish_and_clear();
            println!("  Investment not found.");
        }
    }

    Ok(())
}

// ── Delete investment ─────────────────────────────────────────────────────────

fn interactive_delete(theme: &ColorfulTheme) -> Result<()> {
    println!("  🗑️   Delete an Investment");
    println!("  {}", "─".repeat(30));

    let id = match pick_investment(theme, "Select investment to delete")? {
        Some(id) => id,
        None => return Ok(()),
    };

    // Load the investment to show its name in the confirmation prompt
    let storage = Storage::open();
    let inv = match storage.get_investment(&id)? {
        Some(i) => i,
        None => {
            println!("  Investment not found.");
            return Ok(());
        }
    };

    println!();
    println!("  ⚠  You are about to permanently delete:");
    println!("     {} — {} — invested ${:.2}", inv.name, inv.investment_type, inv.amount);
    println!();

    if !Confirm::with_theme(theme)
        .with_prompt("Are you sure?")
        .default(false)
        .interact()?
    {
        println!("  Cancelled.");
        return Ok(());
    }

    let pb = spinner("Deleting investment…");
    storage.delete_investment(&id)?;
    pb.finish_and_clear();
    println!("  ✓ Deleted: {} ({})", inv.name, inv.id);

    Ok(())
}
