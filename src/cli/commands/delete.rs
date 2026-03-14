use crate::core::Storage;
use crate::error::{InvestmentError, Result};
use crate::utils::display::{fmt_amount, load_currency_symbol, spinner};
use dialoguer::{Confirm, theme::ColorfulTheme};
use std::io::IsTerminal;

pub fn run(id: String, yes: bool) -> Result<()> {
    let storage = Storage::open();

    let inv = match storage.get_investment(&id)? {
        Some(i) => i,
        None => {
            return Err(InvestmentError::NotFound(format!(
                "Investment with ID '{}' not found",
                id
            ))
            .into());
        }
    };

    let cur = load_currency_symbol();
    println!("  ⚠  You are about to permanently delete:");
    println!(
        "     {} — {} — invested {}",
        inv.name,
        inv.investment_type,
        fmt_amount(&cur, inv.amount)
    );
    println!();

    if !yes {
        // Only show the interactive prompt when we are actually attached to a
        // terminal.  In non-TTY contexts (CI, piped usage, tests) dialoguer
        // would hard-error, so we treat the absence of a TTY as "not
        // confirmed" — the caller must pass --yes to delete non-interactively.
        if !std::io::stdin().is_terminal() {
            println!("Cancelled.");
            return Ok(());
        }

        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Are you sure?")
            .default(false)
            .interact()?;

        if !confirmed {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let pb = spinner("Deleting investment…");
    storage.delete_investment(&id)?;
    pb.finish_and_clear();
    println!("✓ Deleted investment: {} ({})", inv.id, inv.name);

    Ok(())
}
