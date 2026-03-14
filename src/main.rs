use clap::{Parser, Subcommand};
use anyhow::Result;

mod models;
mod storage;
use models::{Investment, InvestmentType};
use storage::Storage;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "investment-tracker")]
#[command(version = "0.1.0")]
#[command(about = "A CLI investment tracker for managing your portfolio", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new investment to your portfolio
    Add {
        /// Type of investment (stock, etf, deposit, etc.)
        investment_type: String,
        /// Name or symbol of the investment
        name: String,
        /// Amount invested
        amount: f64,
        /// Date of investment (YYYY-MM-DD)
        date: Option<String>,
    },
    /// List all investments in your portfolio
    List,
    /// View details of a specific investment
    View {
        /// ID of the investment to view
        id: String,
    },
    /// Update an existing investment
    Update {
        /// ID of the investment to update
        id: String,
        /// New amount invested
        amount: Option<f64>,
        /// New date of investment (YYYY-MM-DD)
        date: Option<String>,
    },
    /// Delete an investment from your portfolio
    Delete {
        /// ID of the investment to delete
        id: String,
    },
}

fn parse_investment_type(investment_type: &str) -> Result<InvestmentType> {
    match investment_type.to_lowercase().as_str() {
        "stock" => Ok(InvestmentType::Stock),
        "etf" => Ok(InvestmentType::ETF),
        "mutualfund" | "mutual_fund" => Ok(InvestmentType::MutualFund),
        "deposit" => Ok(InvestmentType::Deposit),
        "bond" => Ok(InvestmentType::Bond),
        "crypto" => Ok(InvestmentType::Crypto),
        other => Ok(InvestmentType::Other(other.to_string())),
    }
}

fn get_storage() -> Storage {
    // Check for environment variable override (for testing)
    if let Ok(data_path) = std::env::var("INVESTMENT_TRACKER_DATA") {
        return Storage::new(PathBuf::from(data_path));
    }
    
    // Default location
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let data_dir = home_dir.join(".investment_tracker");
    let data_file = data_dir.join("investments.json");
    Storage::new(data_file)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let storage = get_storage();
    
    match &cli.command {
        Commands::Add { investment_type, name, amount, date } => {
            let investment_type = parse_investment_type(investment_type)?;
            let date = date.clone().unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
            
            let investment = Investment::new(
                String::new(), // ID will be generated
                investment_type,
                name.clone(),
                None, // No symbol for now
                *amount,
                date,
                None, // No current value initially
                None, // No notes initially
            )?;
            
            let saved_investment = storage.add_investment(investment)?;
            println!("✓ Added investment: {} ({})", saved_investment.name, saved_investment.id);
            println!("  Type: {}", saved_investment.investment_type);
            println!("  Amount: ${:.2}", saved_investment.amount);
            println!("  Date: {}", saved_investment.date);
        }
        Commands::List => {
            let investments = storage.get_all_investments()?;
            
            if investments.is_empty() {
                println!("No investments found.");
                return Ok(());
            }
            
            println!("Your Investments:");
            println!("----------------");
            for investment in &investments {
                println!("{}: {} - ${:.2} ({})", 
                    investment.id, 
                    investment.name, 
                    investment.amount, 
                    investment.date);
            }
            println!("----------------");
            println!("Total: {} investments", investments.len());
        }
        Commands::View { id } => {
            match storage.get_investment(id)? {
                Some(investment) => {
                    println!("Investment Details:");
                    println!("------------------");
                    println!("ID: {}", investment.id);
                    println!("Type: {}", investment.investment_type);
                    println!("Name: {}", investment.name);
                    println!("Amount: ${:.2}", investment.amount);
                    println!("Date: {}", investment.date);
                    
                    if let Some(symbol) = &investment.symbol {
                        println!("Symbol: {}", symbol);
                    }
                    
                    if let Some(current_value) = investment.current_value {
                        println!("Current Value: ${:.2}", current_value);
                        if let Some(roi) = investment.return_on_investment() {
                            println!("Return: ${:.2} ({:.2}%)", roi, investment.return_percentage().unwrap_or(0.0));
                        }
                    }
                    
                    if let Some(notes) = investment.notes {
                        println!("Notes: {}", notes);
                    }
                    
                    println!("Created: {}", investment.created_at);
                    println!("Updated: {}", investment.updated_at);
                }
                None => {
                    println!("Investment with ID '{}' not found.", id);
                }
            }
        }
        Commands::Update { id, amount, date } => {
            match storage.get_investment(id)? {
                Some(mut investment) => {
                    let mut updated = false;
                    
                    if let Some(new_amount) = amount {
                        investment.update_amount(*new_amount)?;
                        updated = true;
                    }
                    
                    if let Some(new_date) = date {
                        investment.date = new_date.clone();
                        updated = true;
                    }
                    
                    if updated {
                        storage.update_investment(&investment)?;
                        println!("✓ Updated investment: {}", investment.id);
                    } else {
                        println!("No changes made to investment: {}", investment.id);
                    }
                }
                None => {
                    println!("Investment with ID '{}' not found.", id);
                }
            }
        }
        Commands::Delete { id } => {
            match storage.delete_investment(id)? {
                Some(deleted) => {
                    println!("✓ Deleted investment: {} ({})", deleted.id, deleted.name);
                }
                None => {
                    println!("Investment with ID '{}' not found.", id);
                }
            }
        }
    }
    
    Ok(())
}
