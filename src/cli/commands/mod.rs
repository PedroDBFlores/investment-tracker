use crate::error::Result;
use clap::{Parser, Subcommand};

pub mod add;
pub mod add_dividend;
pub mod add_price;
pub mod analytics;
pub mod config;
pub mod delete;
pub mod export;
pub mod import;
pub mod interactive;
pub mod list;
pub mod list_dividends;
pub mod performance;
pub mod portfolio;
pub mod sell;
pub mod update;
pub mod view;

#[derive(Parser)]
#[command(name = "investment-tracker")]
#[command(version)]
#[command(about = "A CLI investment tracker for managing your portfolio", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        match self.command {
            Commands::Add {
                investment_type,
                name,
                amount,
                date,
                symbol,
                notes,
                dividend_yield,
                dividend_frequency,
                units,
            } => add::run(
                investment_type,
                name,
                amount,
                date,
                symbol,
                notes,
                dividend_yield,
                dividend_frequency,
                units,
            ),
            Commands::List { limit, offset } => list::run(limit, offset),
            Commands::View { id } => view::run(id),
            Commands::Update {
                id,
                amount,
                current_value,
                date,
                notes,
                units,
            } => update::run(id, amount, current_value, date, notes, units),
            Commands::Delete { id, yes } => delete::run(id, yes),
            Commands::Portfolio { detailed: _ } => portfolio::run(),
            Commands::Export { path, format } => export::run(path, format),
            Commands::Import { path } => import::run(path),
            Commands::Config { command } => config::run(command),
            Commands::AddPrice {
                id,
                price,
                date,
                notes,
                unit_price,
            } => add_price::run(id, price, date, notes, unit_price),
            Commands::Performance { id, range } => performance::run(id, range),
            Commands::AddDividend {
                id,
                amount,
                date,
                notes,
            } => add_dividend::run(id, amount, date, notes),
            Commands::ListDividends { id } => list_dividends::run(id),
            Commands::Analytics => analytics::run(),
            Commands::Interactive => interactive::run(),
            Commands::Sell {
                id,
                units_sold,
                price_per_unit,
                date,
                notes,
            } => sell::run(id, units_sold, price_per_unit, date, notes),
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
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
        /// Ticker symbol (e.g. AAPL, BTC)
        #[arg(short = 's', long)]
        symbol: Option<String>,
        /// Optional notes for this investment
        #[arg(short = 'n', long)]
        notes: Option<String>,
        /// Dividend yield (e.g., 3.5 for 3.5%)
        #[arg(short = 'y', long)]
        dividend_yield: Option<f64>,
        /// Dividend frequency (e.g., "monthly", "quarterly", "annual")
        #[arg(short = 'f', long)]
        dividend_frequency: Option<String>,
        /// Number of units/shares purchased
        #[arg(short = 'u', long)]
        units: Option<f64>,
    },
    /// List all investments in your portfolio
    List {
        /// Maximum number of investments to show (shows all if omitted)
        #[arg(short = 'n', long)]
        limit: Option<usize>,
        /// Number of investments to skip (for pagination)
        #[arg(short, long, default_value_t = 0)]
        offset: usize,
    },
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
        /// New current value
        current_value: Option<f64>,
        /// New date of investment (YYYY-MM-DD)
        date: Option<String>,
        /// Update notes for this investment
        #[arg(short = 'n', long)]
        notes: Option<String>,
        /// Update number of units/shares held
        #[arg(short = 'u', long)]
        units: Option<f64>,
    },
    /// Delete an investment from your portfolio
    Delete {
        /// ID of the investment to delete
        id: String,
        /// Skip the confirmation prompt and delete immediately
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Show portfolio summary and analytics
    Portfolio {
        /// Show detailed allocation breakdown
        #[arg(short, long)]
        detailed: bool,
    },
    /// Export investments to CSV or JSON format
    Export {
        /// Output file path
        path: String,
        /// Export format (csv or json)
        #[arg(short, long, default_value = "csv")]
        format: String,
    },
    /// Import investments from CSV or JSON file
    Import {
        /// Input file path
        path: String,
    },
    /// Manage configuration settings
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Record a new price data point for an investment
    AddPrice {
        /// ID of the investment
        id: String,
        /// New price / current value
        price: f64,
        /// Date of the price entry (YYYY-MM-DD, defaults to today)
        date: Option<String>,
        /// Optional notes
        #[arg(short, long)]
        notes: Option<String>,
        /// Price per individual unit/share at this date
        #[arg(long)]
        unit_price: Option<f64>,
    },
    /// Show performance report for all investments or a specific one
    Performance {
        /// Optional investment ID to show performance for a specific investment
        id: Option<String>,
        /// Time range filter: 1m, 3m, 6m, 1y, all (default: all)
        #[arg(short, long, default_value = "all")]
        range: String,
    },
    /// Record a dividend payment for an investment
    AddDividend {
        /// ID of the investment
        id: String,
        /// Dividend amount received
        amount: f64,
        /// Date received (YYYY-MM-DD, defaults to today)
        date: Option<String>,
        /// Optional notes
        #[arg(short, long)]
        notes: Option<String>,
    },
    /// List dividend history for an investment
    ListDividends {
        /// ID of the investment
        id: String,
    },
    /// Show advanced analytics: best/worst performers, standard deviation of returns
    Analytics,
    /// Launch interactive mode — a guided menu for all portfolio actions
    Interactive,
    /// Record the sale of units in an investment
    Sell {
        /// ID of the investment
        id: String,
        /// Number of units/shares sold
        units_sold: f64,
        /// Sale price per unit/share
        price_per_unit: f64,
        /// Date of sale (YYYY-MM-DD, defaults to today)
        date: Option<String>,
        /// Optional notes
        #[arg(short, long)]
        notes: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set {
        /// Configuration key (data-directory, currency, date-format, show-dividends, color-output)
        key: String,
        /// Configuration value
        value: String,
    },
    /// Reset configuration to defaults
    Reset,
}
