use crate::core::models::validate_date;
use crate::core::{Investment, InvestmentType, Storage};
use crate::error::Result;
use crate::utils::display::{fmt_amount, load_currency_symbol, spinner};
use chrono::Local;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};

// ── Top-level menu actions ────────────────────────────────────────────────────

enum MenuAction {
    AddInvestment,
    ListInvestments,
    UpdateInvestment,
    RecordPrice,
    RecordDividend,
    DeleteInvestment,
    ViewPortfolio,
    Performance,
    Analytics,
    Export,
    Import,
    Config,
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
            MenuAction::ViewPortfolio => "📊  Portfolio summary",
            MenuAction::Performance => "📈  Performance report",
            MenuAction::Analytics => "🔬  Analytics",
            MenuAction::Export => "📤  Export portfolio",
            MenuAction::Import => "📥  Import portfolio",
            MenuAction::Config => "⚙️   Configuration",
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
            MenuAction::ViewPortfolio,
            MenuAction::Performance,
            MenuAction::Analytics,
            MenuAction::Export,
            MenuAction::Import,
            MenuAction::Config,
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

        // Drain any keystrokes (e.g. the Enter from a preceding Confirm prompt)
        // that may still be sitting in the tty input queue.  dialoguer uses the
        // raw tty directly, so a stale \n would otherwise be consumed
        // immediately by the next Select, auto-selecting item 0.
        drain_tty_input();

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
            MenuAction::ViewPortfolio => crate::cli::commands::portfolio::run()?,
            MenuAction::Performance => interactive_performance(&theme)?,
            MenuAction::Analytics => crate::cli::commands::analytics::run()?,
            MenuAction::Export => interactive_export(&theme)?,
            MenuAction::Import => interactive_import(&theme)?,
            MenuAction::Config => interactive_config(&theme)?,
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

// ── TTY input drain ───────────────────────────────────────────────────────────

/// Discard any bytes already waiting in the terminal input queue before we
/// render the main-menu `Select`.  This prevents a stale Enter (left over from
/// a sub-command's final confirmation prompt) from immediately confirming item 0.
///
/// We put the tty in non-blocking mode, read until `EAGAIN`/`EWOULDBLOCK`, then
/// restore blocking mode.  The whole operation is best-effort; any error is
/// silently ignored so it never interrupts the normal flow.
#[cfg(unix)]
fn drain_tty_input() {
    use std::fs::OpenOptions;
    use std::io::Read;
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::io::AsRawFd;

    // Open the controlling terminal directly so we never touch stdin/stdout.
    let Ok(mut tty) = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open("/dev/tty")
    else {
        return;
    };

    let mut buf = [0u8; 64];
    // Keep reading until there is nothing left (EAGAIN) or an error occurs.
    loop {
        match tty.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(_) => continue,
        }
    }

    // Restore blocking mode so dialoguer's subsequent read works normally.
    // SAFETY: fd is valid for the lifetime of `tty`.
    unsafe {
        let fd = tty.as_raw_fd();
        let flags = libc::fcntl(fd, libc::F_GETFL, 0);
        if flags != -1 {
            libc::fcntl(fd, libc::F_SETFL, flags & !libc::O_NONBLOCK);
        }
    }
}

#[cfg(not(unix))]
fn drain_tty_input() {
    // On non-Unix platforms there is no /dev/tty; do nothing.
}

// ── Add investment ────────────────────────────────────────────────────────────

fn interactive_add(theme: &ColorfulTheme) -> Result<()> {
    println!("  ➕  Add a New Investment");
    println!("  {}", "─".repeat(30));

    // Investment type
    let type_labels = vec![
        "Stock",
        "ETF",
        "Mutual Fund",
        "Deposit",
        "Bond",
        "Crypto",
        "Other",
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
        match validate_date(&d) {
            Ok(_) => break d,
            Err(e) => println!("  ⚠  {}", e),
        }
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
        let freq_options = vec![
            "Monthly",
            "Quarterly",
            "Semi-annual",
            "Annual",
            "Skip (no frequency)",
        ];
        let freq_idx = Select::with_theme(theme)
            .with_prompt("Dividend frequency")
            .items(&freq_options)
            .default(1)
            .interact()?;
        match freq_idx {
            0 => Some("Monthly".to_string()),
            1 => Some("Quarterly".to_string()),
            2 => Some("Semi-annual".to_string()),
            3 => Some("Annual".to_string()),
            _ => None,
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
    println!(
        "    Amount: {}",
        fmt_amount(&load_currency_symbol(), amount)
    );
    println!("    Date:   {}", date);
    if let Some(ref n) = notes {
        println!("    Notes:  {}", n);
    }
    if let Some(y) = dividend_yield {
        println!("    Dividend Yield: {:.2}%", y);
    }
    if let Some(ref f) = dividend_frequency {
        println!("    Dividend Freq:  {}", f);
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
        Some(amount),
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

    let cur = load_currency_symbol();

    println!("  📋  Your Investments  ({} total)", investments.len());
    println!("  {}", "─".repeat(60));

    for (i, inv) in investments.iter().enumerate() {
        let short_id = &inv.id[..8.min(inv.id.len())];
        let value_str = match inv.current_value {
            Some(cv) => {
                let roi = cv - inv.amount;
                let sign = if roi >= 0.0 { "+" } else { "" };
                let pct = roi / inv.amount * 100.0;
                format!("{}  ({}{:.1}%)", fmt_amount(&cur, cv), sign, pct)
            }
            None => "—".to_string(),
        };
        println!(
            "  {:>2}.  [{}…]  {}  |  {}  |  invested {}  |  current {}",
            i + 1,
            short_id,
            inv.name,
            inv.investment_type,
            fmt_amount(&cur, inv.amount),
            value_str,
        );
    }

    let summary = portfolio_summary(&investments);

    println!("  {}", "─".repeat(60));
    println!(
        "  💰  Total invested:      {}",
        fmt_amount(&cur, summary.total_invested)
    );
    match summary.total_current {
        Some(cv) => {
            let roi = cv - summary.total_invested;
            let sign = if roi >= 0.0 { "+" } else { "" };
            let pct = roi / summary.total_invested * 100.0;
            println!(
                "  📈  Total current value: {}  ({}{:.1}%)",
                fmt_amount(&cur, cv),
                sign,
                pct
            );
        }
        None => {
            println!("  📈  Total current value: —");
        }
    }

    Ok(())
}

// ── Portfolio summary ─────────────────────────────────────────────────────────

struct PortfolioSummary {
    total_invested: f64,
    total_current: Option<f64>,
}

fn portfolio_summary(investments: &[Investment]) -> PortfolioSummary {
    let total_invested: f64 = investments.iter().map(|inv| inv.amount).sum();
    let current_values: Vec<f64> = investments
        .iter()
        .filter_map(|inv| inv.current_value)
        .collect();
    let total_current = if current_values.is_empty() {
        None
    } else {
        Some(current_values.iter().sum())
    };
    PortfolioSummary {
        total_invested,
        total_current,
    }
}

// ── Pick an investment by interactive list ────────────────────────────────────

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Investment, InvestmentType};

    fn make_investment(amount: f64, current_value: Option<f64>) -> Investment {
        let mut inv = Investment::new(
            uuid::Uuid::new_v4().to_string(),
            InvestmentType::Stock,
            format!("Inv {}", amount),
            None,
            amount,
            "2024-01-01".to_string(),
            None,
            None,
            None,
            None,
        )
        .expect("valid investment");
        inv.current_value = current_value;
        inv
    }

    // ── portfolio_summary ─────────────────────────────────────────────────────

    #[test]
    fn test_summary_empty_list() {
        let summary = portfolio_summary(&[]);
        assert_eq!(summary.total_invested, 0.0);
        assert!(summary.total_current.is_none());
    }

    #[test]
    fn test_summary_no_current_values() {
        let investments = vec![make_investment(1000.0, None), make_investment(2000.0, None)];
        let summary = portfolio_summary(&investments);
        assert_eq!(summary.total_invested, 3000.0);
        assert!(summary.total_current.is_none());
    }

    #[test]
    fn test_summary_all_have_current_values() {
        let investments = vec![
            make_investment(1000.0, Some(1200.0)),
            make_investment(2000.0, Some(2500.0)),
        ];
        let summary = portfolio_summary(&investments);
        assert_eq!(summary.total_invested, 3000.0);
        assert_eq!(summary.total_current, Some(3700.0));
    }

    #[test]
    fn test_summary_partial_current_values() {
        // Only investments with a current_value should be summed into total_current
        let investments = vec![
            make_investment(1000.0, Some(1100.0)),
            make_investment(500.0, None),
            make_investment(2000.0, Some(1800.0)),
        ];
        let summary = portfolio_summary(&investments);
        assert_eq!(summary.total_invested, 3500.0);
        assert_eq!(summary.total_current, Some(2900.0));
    }

    #[test]
    fn test_summary_single_investment_with_gain() {
        let investments = vec![make_investment(1000.0, Some(1500.0))];
        let summary = portfolio_summary(&investments);
        assert_eq!(summary.total_invested, 1000.0);
        assert_eq!(summary.total_current, Some(1500.0));
        // Verify the derived ROI and percentage are correct
        let cv = summary.total_current.unwrap();
        let roi = cv - summary.total_invested;
        let pct = roi / summary.total_invested * 100.0;
        assert!((roi - 500.0).abs() < f64::EPSILON);
        assert!((pct - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_summary_single_investment_with_loss() {
        let investments = vec![make_investment(2000.0, Some(1600.0))];
        let summary = portfolio_summary(&investments);
        assert_eq!(summary.total_invested, 2000.0);
        assert_eq!(summary.total_current, Some(1600.0));
        let cv = summary.total_current.unwrap();
        let roi = cv - summary.total_invested;
        let pct = roi / summary.total_invested * 100.0;
        assert!((roi - (-400.0)).abs() < f64::EPSILON);
        assert!((pct - (-20.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_summary_totals_are_sum_of_all_investments() {
        let investments = vec![
            make_investment(500.0, Some(600.0)),
            make_investment(1500.0, Some(1400.0)),
            make_investment(3000.0, Some(3300.0)),
        ];
        let summary = portfolio_summary(&investments);
        assert_eq!(summary.total_invested, 5000.0);
        assert_eq!(summary.total_current, Some(5300.0));
    }

    // ── Issue 1: current_value defaults to amount on creation ─────────────────

    /// A freshly created investment must have current_value == amount.
    #[test]
    fn test_new_investment_current_value_equals_amount() {
        let inv = Investment::new(
            uuid::Uuid::new_v4().to_string(),
            InvestmentType::Stock,
            "Test Corp".to_string(),
            None,
            1500.0,
            "2024-06-01".to_string(),
            Some(1500.0), // current_value set to amount at creation
            None,
            None,
            None,
        )
        .expect("valid investment");

        assert_eq!(
            inv.current_value,
            Some(inv.amount),
            "current_value should equal amount immediately after creation"
        );
    }

    /// Verifies that the value passed as current_value is exactly the amount,
    /// not None and not some other value.
    #[test]
    fn test_new_investment_current_value_is_not_none() {
        let amount = 2750.0;
        let inv = Investment::new(
            uuid::Uuid::new_v4().to_string(),
            InvestmentType::ETF,
            "My ETF".to_string(),
            Some("ETFX".to_string()),
            amount,
            "2024-01-15".to_string(),
            Some(amount),
            None,
            None,
            None,
        )
        .expect("valid investment");

        assert!(
            inv.current_value.is_some(),
            "current_value must not be None after creation"
        );
        assert_eq!(inv.current_value.unwrap(), amount);
    }

    /// Updating the current_value after creation must NOT change the original
    /// invested amount — the two are independent once the investment exists.
    #[test]
    fn test_current_value_can_diverge_from_amount_after_update() {
        let amount = 1000.0;
        let mut inv = Investment::new(
            uuid::Uuid::new_v4().to_string(),
            InvestmentType::Stock,
            "Diverge Corp".to_string(),
            None,
            amount,
            "2024-03-01".to_string(),
            Some(amount),
            None,
            None,
            None,
        )
        .expect("valid investment");

        // Simulate a later price update
        let _ = inv.update_current_value(1250.0);

        assert_eq!(inv.amount, amount, "invested amount must stay unchanged");
        assert_eq!(
            inv.current_value,
            Some(1250.0),
            "current_value should reflect the updated price"
        );
    }

    /// A new investment with current_value == amount shows 0 ROI and 0%.
    #[test]
    fn test_new_investment_has_zero_return_on_creation() {
        let amount = 500.0;
        let inv = Investment::new(
            uuid::Uuid::new_v4().to_string(),
            InvestmentType::Deposit,
            "Zero Return".to_string(),
            None,
            amount,
            "2024-01-01".to_string(),
            Some(amount),
            None,
            None,
            None,
        )
        .expect("valid investment");

        let roi = inv.return_on_investment().expect("ROI should be available");
        let pct = inv
            .return_percentage()
            .expect("percentage should be available");
        assert!(
            roi.abs() < f64::EPSILON,
            "ROI should be 0 when current == invested"
        );
        assert!(
            pct.abs() < f64::EPSILON,
            "return % should be 0 when current == invested"
        );
    }

    // ── Issue 2: drain_tty_input must not panic ────────────────────────────────

    /// `drain_tty_input` is a best-effort, side-effect-only function.
    /// The only hard requirement is that it never panics, regardless of the
    /// terminal environment (e.g. in a CI runner with no tty attached).
    #[test]
    fn test_drain_tty_input_does_not_panic() {
        // Call it twice to ensure repeated calls are also safe.
        drain_tty_input();
        drain_tty_input();
    }
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
            let cur = load_currency_symbol();
            let value_str = inv
                .current_value
                .map(|cv| format!("  current {}", fmt_amount(&cur, cv)))
                .unwrap_or_default();
            format!(
                "[{}…]  {}  (invested {}{})",
                short_id,
                inv.name,
                fmt_amount(&cur, inv.amount),
                value_str
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
    let cur = load_currency_symbol();
    let amount_str: String = Input::with_theme(theme)
        .with_prompt(format!(
            "New amount invested (current: {})",
            fmt_amount(&cur, inv.amount)
        ))
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
                .map(|v| fmt_amount(&cur, v))
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
        match validate_date(&d) {
            Ok(_) => inv.date = d,
            Err(e) => println!("  ⚠  {} — skipped.", e),
        }
    }

    // Notes
    let current_notes = inv.notes.clone().unwrap_or_else(|| "—".to_string());
    let notes_str: String = Input::with_theme(theme)
        .with_prompt(format!("Notes (current: {})", current_notes))
        .allow_empty(true)
        .interact_text()?;
    if !notes_str.trim().is_empty() {
        inv.notes = Some(notes_str.trim().to_string());
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
        match validate_date(&d) {
            Ok(_) => break d,
            Err(e) => println!("  ⚠  {}", e),
        }
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
            println!(
                "  ✓ Recorded price {} for {} on {}",
                fmt_amount(&load_currency_symbol(), price),
                inv.name,
                date
            );
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
        match validate_date(&d) {
            Ok(_) => break d,
            Err(e) => println!("  ⚠  {}", e),
        }
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
            let cur = load_currency_symbol();
            println!(
                "  ✓ Recorded dividend {} for {} on {}",
                fmt_amount(&cur, amount),
                inv.name,
                date
            );
            println!(
                "    Total dividends: {}",
                fmt_amount(&cur, inv.total_dividends())
            );
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
    println!(
        "     {} — {} — invested {}",
        inv.name,
        inv.investment_type,
        fmt_amount(&load_currency_symbol(), inv.amount)
    );
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

// ── Performance report ────────────────────────────────────────────────────────

fn interactive_performance(theme: &ColorfulTheme) -> Result<()> {
    println!("  📈  Performance Report");
    println!("  {}", "─".repeat(30));

    let range_options = vec!["all", "1m", "3m", "6m", "1y"];
    let range_idx = Select::with_theme(theme)
        .with_prompt("Time range")
        .items(&range_options)
        .default(0)
        .interact()?;
    let range = range_options[range_idx].to_string();

    let target_options = vec!["All investments", "A specific investment"];
    let target_idx = Select::with_theme(theme)
        .with_prompt("Show performance for")
        .items(&target_options)
        .default(0)
        .interact()?;

    let id = if target_idx == 1 {
        pick_investment(theme, "Select investment")?
    } else {
        None
    };

    println!();
    crate::cli::commands::performance::run(id, range)
}

// ── Export ────────────────────────────────────────────────────────────────────

fn interactive_export(theme: &ColorfulTheme) -> Result<()> {
    println!("  📤  Export Portfolio");
    println!("  {}", "─".repeat(30));

    let format_options = vec!["csv", "json"];
    let format_idx = Select::with_theme(theme)
        .with_prompt("Export format")
        .items(&format_options)
        .default(0)
        .interact()?;
    let format = format_options[format_idx].to_string();

    let default_name = format!(
        "portfolio_{}.{}",
        chrono::Local::now().format("%Y%m%d"),
        format
    );
    let path: String = Input::with_theme(theme)
        .with_prompt("Output file path")
        .default(default_name)
        .interact_text()?;

    println!();
    crate::cli::commands::export::run(path, format)
}

// ── Import ────────────────────────────────────────────────────────────────────

fn interactive_import(theme: &ColorfulTheme) -> Result<()> {
    println!("  📥  Import Portfolio");
    println!("  {}", "─".repeat(30));

    let path: String = Input::with_theme(theme)
        .with_prompt("Input file path (CSV or JSON)")
        .interact_text()?;

    println!();
    if !Confirm::with_theme(theme)
        .with_prompt(format!("Import investments from '{}'?", path))
        .default(true)
        .interact()?
    {
        println!("  Cancelled.");
        return Ok(());
    }

    println!();
    crate::cli::commands::import::run(path)
}

// ── Config ────────────────────────────────────────────────────────────────────

fn interactive_config(theme: &ColorfulTheme) -> Result<()> {
    println!("  ⚙️   Configuration");
    println!("  {}", "─".repeat(30));

    use super::ConfigCommands;

    let action_options = vec![
        "Show current settings",
        "Change a setting",
        "Reset to defaults",
    ];
    let action_idx = Select::with_theme(theme)
        .with_prompt("What would you like to do?")
        .items(&action_options)
        .default(0)
        .interact()?;

    println!();

    match action_idx {
        0 => crate::cli::commands::config::run(ConfigCommands::Show),
        1 => {
            let key_options = vec![
                "currency",
                "data-directory",
                "date-format",
                "show-dividends",
                "color-output",
            ];
            let key_idx = Select::with_theme(theme)
                .with_prompt("Which setting?")
                .items(&key_options)
                .default(0)
                .interact()?;
            let key = key_options[key_idx].to_string();

            let value: String = Input::with_theme(theme)
                .with_prompt(format!("New value for '{}'", key))
                .interact_text()?;

            println!();
            crate::cli::commands::config::run(ConfigCommands::Set { key, value })
        }
        _ => {
            if !Confirm::with_theme(theme)
                .with_prompt("Reset all configuration to defaults?")
                .default(false)
                .interact()?
            {
                println!("  Cancelled.");
                return Ok(());
            }
            println!();
            crate::cli::commands::config::run(ConfigCommands::Reset)
        }
    }
}
