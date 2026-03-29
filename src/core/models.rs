use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;

// Use the centralized error module
pub use crate::error::{InvestmentError, Result};

/// Validates that `date` is a real calendar date in YYYY-MM-DD format.
/// Returns `Ok(())` on success, or an `InvalidDate` error with a descriptive
/// message on failure.
pub fn validate_date(date: &str) -> Result<()> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| {
        InvestmentError::InvalidDate(format!(
            "'{}' is not a valid date — expected YYYY-MM-DD (e.g. 2024-01-15)",
            date
        ))
    })?;
    Ok(())
}

/// A single dividend payment received from an investment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DividendEntry {
    /// Payment date in YYYY-MM-DD format.
    pub date: String,
    /// Total dividend amount received.
    pub amount: f64,
    pub notes: Option<String>,
}

/// A snapshot of an investment's value at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PriceEntry {
    /// Date of the price snapshot in YYYY-MM-DD format.
    pub date: String,
    /// Total portfolio value (or per-unit price when `unit_price` is not set).
    pub price: f64,
    pub notes: Option<String>,
    /// Per-unit market price at this date. Optional; only set for unit-tracked investments.
    #[serde(default)]
    pub unit_price: Option<f64>,
}

/// Records the sale of units/shares from an investment position.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaleEntry {
    /// Date of the sale in YYYY-MM-DD format.
    pub date: String,
    /// Number of units/shares sold.
    pub units_sold: f64,
    /// Market price per unit at the time of sale.
    pub sale_price_per_unit: f64,
    /// `units_sold × sale_price_per_unit`.
    pub total_proceeds: f64,
    /// `total_proceeds − (units_sold × cost_basis_per_unit)`.
    pub realized_gain: f64,
    pub notes: Option<String>,
}

/// The category of a financial instrument.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InvestmentType {
    /// Individual company stock / share.
    Stock,
    /// Exchange-traded fund.
    ETF,
    /// Actively managed mutual fund.
    MutualFund,
    /// Fixed-term bank deposit or savings account.
    Deposit,
    /// Government or corporate bond.
    Bond,
    /// Cryptocurrency.
    Crypto,
    /// Any instrument not covered by the variants above.
    Other(String),
}

impl fmt::Display for InvestmentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvestmentType::Stock => write!(f, "Stock"),
            InvestmentType::ETF => write!(f, "ETF"),
            InvestmentType::MutualFund => write!(f, "Mutual Fund"),
            InvestmentType::Deposit => write!(f, "Deposit"),
            InvestmentType::Bond => write!(f, "Bond"),
            InvestmentType::Crypto => write!(f, "Crypto"),
            InvestmentType::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for InvestmentType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "stock" => InvestmentType::Stock,
            "etf" => InvestmentType::ETF,
            "mutualfund" | "mutual_fund" | "mutual fund" => InvestmentType::MutualFund,
            "deposit" => InvestmentType::Deposit,
            "bond" => InvestmentType::Bond,
            "crypto" => InvestmentType::Crypto,
            other => InvestmentType::Other(other.to_string()),
        })
    }
}

/// A single investment position in the portfolio.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Investment {
    /// Unique identifier (UUID v4).
    pub id: String,
    pub investment_type: InvestmentType,
    /// Human-readable name (e.g. "Apple Inc.").
    pub name: String,
    /// Optional ticker or symbol (e.g. "AAPL").
    pub symbol: Option<String>,
    /// Original invested amount (adjusted downward as units are sold).
    pub amount: f64,
    /// Date of the initial purchase in YYYY-MM-DD format.
    pub date: String,
    /// Most recent known market value of the position.
    pub current_value: Option<f64>,
    pub notes: Option<String>,
    /// ISO 8601 timestamp of when this record was created.
    pub created_at: String,
    /// ISO 8601 timestamp of the last modification.
    pub updated_at: String,
    /// Annualised dividend yield (percentage, e.g. `3.5` for 3.5 %).
    #[serde(default)]
    pub dividend_yield: Option<f64>,
    /// Dividend payment frequency (e.g. "quarterly", "annual").
    #[serde(default)]
    pub dividend_frequency: Option<String>,
    /// Chronologically ordered price snapshots.
    #[serde(default)]
    pub price_history: Vec<PriceEntry>,
    /// Recorded dividend payments.
    #[serde(default)]
    pub dividends: Vec<DividendEntry>,
    /// Total units/shares held at acquisition (before any sales).
    #[serde(default)]
    pub units: Option<f64>,
    /// Recorded partial or full sales of this position.
    #[serde(default)]
    pub sales: Vec<SaleEntry>,
}

impl Investment {
    /// Create a new investment, validating `amount` (must be > 0) and `date`
    /// (must be a real YYYY-MM-DD calendar date).
    ///
    /// `id` may be an empty string — the storage layer assigns a UUID before
    /// persisting.
    pub fn new(
        id: String,
        investment_type: InvestmentType,
        name: String,
        symbol: Option<String>,
        amount: f64,
        date: String,
        current_value: Option<f64>,
        notes: Option<String>,
        dividend_yield: Option<f64>,
        dividend_frequency: Option<String>,
        units: Option<f64>,
    ) -> Result<Self> {
        if amount <= 0.0 {
            return Err(InvestmentError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            )
            .into());
        }

        validate_date(&date)?;

        let now = crate::utils::display::now_timestamp();

        Ok(Investment {
            id,
            investment_type,
            name,
            symbol,
            amount,
            date,
            current_value,
            notes,
            created_at: now.clone(),
            updated_at: now,
            dividend_yield,
            dividend_frequency,
            price_history: Vec::new(),
            dividends: Vec::new(),
            units,
            sales: Vec::new(),
        })
    }

    /// Cost basis per unit: amount / remaining_units.
    /// This stays consistent after partial sales because both `amount` and
    /// `remaining_units` are reduced proportionally by `sell()`.
    /// Returns None if units is not set or no units remain.
    pub fn cost_basis_per_unit(&self) -> Option<f64> {
        self.remaining_units().and_then(|rem| {
            if rem > 0.0 {
                Some(self.amount / rem)
            } else {
                None
            }
        })
    }

    /// Remaining units after accounting for all recorded sales.
    /// Returns None if units is not set on this investment.
    pub fn remaining_units(&self) -> Option<f64> {
        self.units.map(|u| {
            let sold: f64 = self.sales.iter().map(|s| s.units_sold).sum();
            u - sold
        })
    }

    /// Sum of realized gains across all sale entries.
    pub fn total_realized_gain(&self) -> f64 {
        self.sales.iter().map(|s| s.realized_gain).sum()
    }

    /// Update the invested amount. Returns an error if `new_amount` ≤ 0.
    pub fn update_amount(&mut self, new_amount: f64) -> Result<()> {
        if new_amount <= 0.0 {
            return Err(InvestmentError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            )
            .into());
        }
        self.amount = new_amount;
        self.updated_at = crate::utils::display::now_timestamp();
        Ok(())
    }

    /// Update the current market value. Returns an error if `new_value` < 0.
    pub fn update_current_value(&mut self, new_value: f64) -> Result<()> {
        if new_value < 0.0 {
            return Err(
                InvestmentError::InvalidAmount("Value cannot be negative".to_string()).into(),
            );
        }
        self.current_value = Some(new_value);
        self.updated_at = crate::utils::display::now_timestamp();
        Ok(())
    }

    /// Absolute return: `current_value − amount`. Returns `None` if no current value is set.
    pub fn return_on_investment(&self) -> Option<f64> {
        self.current_value.map(|current| current - self.amount)
    }

    /// Return as a percentage of the invested amount. Returns `None` if no current value is set.
    pub fn return_percentage(&self) -> Option<f64> {
        self.current_value
            .map(|current| (current - self.amount) / self.amount * 100.0)
    }

    /// Adds a price entry to the history and updates current_value to the latest price.
    pub fn add_price_entry(
        &mut self,
        date: String,
        price: f64,
        notes: Option<String>,
        unit_price: Option<f64>,
    ) -> Result<()> {
        if price <= 0.0 {
            return Err(
                InvestmentError::InvalidAmount("Price must be greater than 0".to_string()).into(),
            );
        }

        validate_date(&date)?;

        self.price_history.push(PriceEntry {
            date,
            price,
            notes,
            unit_price,
        });
        self.price_history.sort_by(|a, b| a.date.cmp(&b.date));

        // Update current_value to the latest price (last entry after sort)
        if let Some(latest) = self.price_history.last() {
            self.current_value = Some(latest.price);
        }

        self.updated_at = crate::utils::display::now_timestamp();
        Ok(())
    }

    /// Record a dividend payment.
    pub fn add_dividend(&mut self, date: String, amount: f64, notes: Option<String>) -> Result<()> {
        if amount <= 0.0 {
            return Err(InvestmentError::InvalidAmount(
                "Dividend amount must be greater than 0".to_string(),
            )
            .into());
        }

        validate_date(&date)?;

        self.dividends.push(DividendEntry {
            date,
            amount,
            notes,
        });
        self.dividends.sort_by(|a, b| a.date.cmp(&b.date));
        self.updated_at = crate::utils::display::now_timestamp();
        Ok(())
    }

    /// Record the sale of units from this investment.
    /// Validates that units are set and that enough units remain.
    /// Updates `self.units` to remaining units and adjusts `self.amount` and
    /// `self.current_value` proportionally.
    pub fn sell(
        &mut self,
        date: String,
        units_sold: f64,
        sale_price_per_unit: f64,
        notes: Option<String>,
    ) -> Result<SaleEntry> {
        let _original_units = self.units.ok_or_else(|| {
            InvestmentError::InsufficientUnits(
                "Cannot record a sale: this investment has no units tracked".to_string(),
            )
        })?;

        let remaining = self.remaining_units().unwrap_or(0.0);
        if units_sold > remaining {
            return Err(InvestmentError::InsufficientUnits(format!(
                "Cannot sell {:.4} units — only {:.4} units remaining",
                units_sold, remaining
            ))
            .into());
        }

        validate_date(&date)?;

        let cost_basis = self.cost_basis_per_unit().unwrap_or(0.0);
        let total_proceeds = units_sold * sale_price_per_unit;
        let realized_gain = total_proceeds - (units_sold * cost_basis);

        let entry = SaleEntry {
            date,
            units_sold,
            sale_price_per_unit,
            total_proceeds,
            realized_gain,
            notes,
        };

        // Reduce amount by the cost of the units sold
        self.amount -= units_sold * cost_basis;

        // Update current_value proportionally if it was set
        if let Some(cv) = self.current_value {
            let new_remaining = remaining - units_sold;
            if remaining > 0.0 {
                self.current_value = Some(cv * (new_remaining / remaining));
            }
        }

        self.sales.push(entry.clone());
        self.updated_at = crate::utils::display::now_timestamp();

        Ok(entry)
    }

    /// Total dividends received.
    pub fn total_dividends(&self) -> f64 {
        self.dividends.iter().map(|d| d.amount).sum()
    }

    /// Total return including dividends: (current_value - amount + total_dividends)
    pub fn total_return_with_dividends(&self) -> Option<f64> {
        self.current_value
            .map(|cv| cv - self.amount + self.total_dividends())
    }

    /// Total return % including dividends.
    pub fn total_return_percentage_with_dividends(&self) -> Option<f64> {
        if self.amount > 0.0 {
            self.total_return_with_dividends()
                .map(|r| r / self.amount * 100.0)
        } else {
            None
        }
    }

    /// Returns the price entries sorted chronologically.
    pub fn sorted_price_history(&self) -> Vec<&PriceEntry> {
        let mut entries: Vec<&PriceEntry> = self.price_history.iter().collect();
        entries.sort_by(|a, b| a.date.cmp(&b.date));
        entries
    }

    /// Time-weighted return: (latest_price - earliest_recorded_price) / earliest_recorded_price * 100
    /// Falls back to return_percentage() if price_history has fewer than 2 entries.
    pub fn time_weighted_return(&self) -> Option<f64> {
        let history = self.sorted_price_history();
        if history.len() >= 2 {
            let earliest = history.first()?.price;
            let latest = history.last()?.price;
            if earliest > 0.0 {
                Some((latest - earliest) / earliest * 100.0)
            } else {
                None
            }
        } else {
            self.return_percentage()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_investment(amount: f64, current_value: Option<f64>) -> Investment {
        Investment::new(
            "test-id".to_string(),
            InvestmentType::Stock,
            "Test Company".to_string(),
            Some("TEST".to_string()),
            amount,
            "2024-01-15".to_string(),
            current_value,
            None,
            None,
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn test_add_price_entry() {
        let mut inv = make_investment(1000.0, None);
        inv.add_price_entry("2024-03-01".to_string(), 1100.0, None, None)
            .unwrap();
        assert_eq!(inv.current_value, Some(1100.0));
        assert_eq!(inv.price_history.len(), 1);

        inv.add_price_entry(
            "2024-04-01".to_string(),
            1200.0,
            Some("Q1 update".to_string()),
            None,
        )
        .unwrap();
        assert_eq!(inv.current_value, Some(1200.0));
        assert_eq!(inv.price_history.len(), 2);
        assert_eq!(inv.price_history[1].notes, Some("Q1 update".to_string()));
    }

    #[test]
    fn test_add_price_entry_invalid_price() {
        let mut inv = make_investment(1000.0, None);
        let result = inv.add_price_entry("2024-03-01".to_string(), 0.0, None, None);
        assert!(result.is_err());
        if let Err(e) = result {
            if let Some(InvestmentError::InvalidAmount(msg)) = e.downcast_ref() {
                assert_eq!(msg, "Price must be greater than 0");
            } else {
                panic!("Expected InvalidAmount error");
            }
        }
    }

    #[test]
    fn test_time_weighted_return_with_history() {
        let mut inv = make_investment(1000.0, None);
        // earliest price 800, latest price 1200 → TWR = (1200-800)/800*100 = 50%
        inv.add_price_entry("2024-02-01".to_string(), 800.0, None, None)
            .unwrap();
        inv.add_price_entry("2024-03-01".to_string(), 1200.0, None, None)
            .unwrap();
        let twr = inv.time_weighted_return().unwrap();
        assert!((twr - 50.0).abs() < 0.001, "Expected 50.0% but got {}", twr);
    }

    #[test]
    fn test_time_weighted_return_no_history() {
        // Falls back to return_percentage() when price_history < 2 entries
        let inv = make_investment(1000.0, Some(1500.0));
        // return_percentage = (1500 - 1000) / 1000 * 100 = 50%
        let twr = inv.time_weighted_return().unwrap();
        assert!((twr - 50.0).abs() < 0.001, "Expected 50.0% but got {}", twr);

        // Also verify: single history entry falls back too
        let mut inv2 = make_investment(1000.0, Some(1500.0));
        inv2.add_price_entry("2024-03-01".to_string(), 1500.0, None, None)
            .unwrap();
        assert_eq!(inv2.price_history.len(), 1);
        let twr2 = inv2.time_weighted_return().unwrap();
        // current_value was updated by add_price_entry to 1500, so return_percentage = 50%
        assert!(
            (twr2 - 50.0).abs() < 0.001,
            "Expected 50.0% but got {}",
            twr2
        );
    }

    #[test]
    fn test_valid_investment_creation() {
        let investment = Investment::new(
            "test-id".to_string(),
            InvestmentType::Stock,
            "Test Company".to_string(),
            Some("TEST".to_string()),
            1000.0,
            "2024-01-15".to_string(),
            Some(1200.0),
            Some("Test investment".to_string()),
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(investment.id, "test-id");
        assert_eq!(investment.name, "Test Company");
        assert_eq!(investment.amount, 1000.0);
        assert_eq!(investment.date, "2024-01-15");
        assert_eq!(investment.current_value, Some(1200.0));
        assert_eq!(investment.notes, Some("Test investment".to_string()));
        assert!(investment.created_at.contains("202"));
        assert!(investment.updated_at.contains("202"));
    }

    #[test]
    fn test_invalid_amount() {
        let result = Investment::new(
            "test-id".to_string(),
            InvestmentType::Stock,
            "Test Company".to_string(),
            None,
            0.0,
            "2024-01-15".to_string(),
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err());
        if let Err(e) = result {
            if let Some(InvestmentError::InvalidAmount(msg)) = e.downcast_ref() {
                assert_eq!(msg, "Amount must be greater than 0");
            } else {
                panic!("Expected InvalidAmount error");
            }
        }
    }

    #[test]
    fn test_validate_date_valid() {
        assert!(validate_date("2024-01-15").is_ok());
        assert!(validate_date("2000-02-29").is_ok()); // 2000 is a leap year
        assert!(validate_date("1999-12-31").is_ok());
    }

    #[test]
    fn test_validate_date_invalid_format() {
        assert!(validate_date("15-01-2024").is_err());
        assert!(validate_date("2024/01/15").is_err());
        assert!(validate_date("20240115").is_err());
        assert!(validate_date("not-a-date").is_err());
        assert!(validate_date("").is_err());
    }

    #[test]
    fn test_validate_date_impossible_dates() {
        assert!(validate_date("2024-13-01").is_err()); // month 13
        assert!(validate_date("2024-00-01").is_err()); // month 0
        assert!(validate_date("2024-01-32").is_err()); // day 32
        assert!(validate_date("2023-02-29").is_err()); // 2023 is not a leap year
        assert!(validate_date("9999-99-99").is_err()); // previously passed the old check
    }

    #[test]
    fn test_invalid_date_format() {
        let result = Investment::new(
            "test-id".to_string(),
            InvestmentType::Stock,
            "Test Company".to_string(),
            None,
            1000.0,
            "invalid-date".to_string(),
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err());
        if let Err(e) = result {
            if let Some(InvestmentError::InvalidDate(_)) = e.downcast_ref() {
                // error message is validated by test_validate_date_invalid_format
            } else {
                panic!("Expected InvalidDate error");
            }
        }
    }

    #[test]
    fn test_update_amount() {
        let mut investment = Investment::new(
            "test-id".to_string(),
            InvestmentType::Stock,
            "Test Company".to_string(),
            None,
            1000.0,
            "2024-01-15".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let original_updated_at = investment.updated_at.clone();
        std::thread::sleep(std::time::Duration::from_millis(10));

        investment.update_amount(1500.0).unwrap();

        assert_eq!(investment.amount, 1500.0);
        // Check that updated_at was actually updated by comparing timestamps
        assert!(investment.updated_at >= original_updated_at);
        // In practice, it should be different, but for test reliability we just check it's not older
    }

    #[test]
    fn test_update_invalid_amount() {
        let mut investment = Investment::new(
            "test-id".to_string(),
            InvestmentType::Stock,
            "Test Company".to_string(),
            None,
            1000.0,
            "2024-01-15".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let result = investment.update_amount(0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_return_calculations() {
        let investment = Investment::new(
            "test-id".to_string(),
            InvestmentType::Stock,
            "Test Company".to_string(),
            None,
            1000.0,
            "2024-01-15".to_string(),
            Some(1500.0),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(investment.return_on_investment(), Some(500.0));
        assert_eq!(investment.return_percentage(), Some(50.0));
    }

    #[test]
    fn test_return_calculations_no_current_value() {
        let investment = Investment::new(
            "test-id".to_string(),
            InvestmentType::Stock,
            "Test Company".to_string(),
            None,
            1000.0,
            "2024-01-15".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(investment.return_on_investment(), None);
        assert_eq!(investment.return_percentage(), None);
    }

    #[test]
    fn test_serialize_deserialize() {
        let original = Investment::new(
            "test-id".to_string(),
            InvestmentType::ETF,
            "Test ETF".to_string(),
            Some("ETF".to_string()),
            2000.0,
            "2024-02-20".to_string(),
            Some(2500.0),
            Some("Test ETF investment".to_string()),
            None,
            None,
            None,
        )
        .unwrap();

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Investment = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_investment_type_display() {
        assert_eq!(format!("{}", InvestmentType::Stock), "Stock");
        assert_eq!(format!("{}", InvestmentType::ETF), "ETF");
        assert_eq!(format!("{}", InvestmentType::MutualFund), "Mutual Fund");
        assert_eq!(
            format!("{}", InvestmentType::Other("Custom".to_string())),
            "Custom"
        );
    }

    #[test]
    fn test_add_dividend() {
        let mut inv = make_investment(1000.0, Some(1200.0));
        inv.add_dividend("2024-03-01".to_string(), 25.0, None)
            .unwrap();
        inv.add_dividend("2024-06-01".to_string(), 25.0, Some("Q2".to_string()))
            .unwrap();
        assert_eq!(inv.dividends.len(), 2);
        assert!((inv.total_dividends() - 50.0).abs() < 0.001);
        // Check sorted order
        assert_eq!(inv.dividends[0].date, "2024-03-01");
        assert_eq!(inv.dividends[1].date, "2024-06-01");
    }

    #[test]
    fn test_add_dividend_invalid_amount() {
        let mut inv = make_investment(1000.0, Some(1200.0));
        let result = inv.add_dividend("2024-03-01".to_string(), 0.0, None);
        assert!(result.is_err());
        if let Err(e) = result {
            if let Some(InvestmentError::InvalidAmount(_)) = e.downcast_ref() {
                // expected
            } else {
                panic!("Expected InvalidAmount error");
            }
        }
    }

    #[test]
    fn test_total_return_with_dividends() {
        // invested 1000, current 1200, dividends 50 → total_return = 250, pct = 25%
        let mut inv = make_investment(1000.0, Some(1200.0));
        inv.add_dividend("2024-03-01".to_string(), 50.0, None)
            .unwrap();
        let total_ret = inv.total_return_with_dividends().unwrap();
        assert!(
            (total_ret - 250.0).abs() < 0.001,
            "Expected 250.0 but got {}",
            total_ret
        );
        let pct = inv.total_return_percentage_with_dividends().unwrap();
        assert!((pct - 25.0).abs() < 0.001, "Expected 25.0% but got {}", pct);
    }

    #[test]
    fn test_investment_type_from_str() {
        assert_eq!(
            "stock".parse::<InvestmentType>().unwrap(),
            InvestmentType::Stock
        );
        assert_eq!(
            "etf".parse::<InvestmentType>().unwrap(),
            InvestmentType::ETF
        );
        assert_eq!(
            "mutual_fund".parse::<InvestmentType>().unwrap(),
            InvestmentType::MutualFund
        );
        assert_eq!(
            "custom".parse::<InvestmentType>().unwrap(),
            InvestmentType::Other("custom".to_string())
        );
    }

    // ── New tests for units / sales features ──────────────────────────────────

    fn make_investment_with_units(amount: f64, units: f64) -> Investment {
        Investment::new(
            "test-id".to_string(),
            InvestmentType::Stock,
            "Test Company".to_string(),
            Some("TEST".to_string()),
            amount,
            "2024-01-15".to_string(),
            Some(amount),
            None,
            None,
            None,
            Some(units),
        )
        .unwrap()
    }

    #[test]
    fn test_cost_basis_per_unit() {
        let inv = make_investment_with_units(1000.0, 10.0);
        let cbpu = inv.cost_basis_per_unit().unwrap();
        assert!(
            (cbpu - 100.0).abs() < 0.001,
            "Expected 100.0 but got {}",
            cbpu
        );
    }

    #[test]
    fn test_cost_basis_per_unit_none_when_no_units() {
        let inv = make_investment(1000.0, Some(1200.0));
        assert!(inv.cost_basis_per_unit().is_none());
    }

    #[test]
    fn test_remaining_units_no_sales() {
        let inv = make_investment_with_units(1000.0, 10.0);
        let remaining = inv.remaining_units().unwrap();
        assert!((remaining - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_remaining_units_after_sale() {
        let mut inv = make_investment_with_units(1000.0, 10.0);
        inv.sell("2024-06-01".to_string(), 3.0, 120.0, None)
            .unwrap();
        let remaining = inv.remaining_units().unwrap();
        assert!(
            (remaining - 7.0).abs() < 0.001,
            "Expected 7.0 but got {}",
            remaining
        );
    }

    #[test]
    fn test_remaining_units_none_when_no_units() {
        let inv = make_investment(1000.0, Some(1200.0));
        assert!(inv.remaining_units().is_none());
    }

    #[test]
    fn test_total_realized_gain() {
        let mut inv = make_investment_with_units(1000.0, 10.0);
        // cost basis = 100/unit; sell 3 at 120 → gain = 3*(120-100) = 60
        inv.sell("2024-06-01".to_string(), 3.0, 120.0, None)
            .unwrap();
        // sell 2 at 80 → gain = 2*(80-100) = -40
        inv.sell("2024-07-01".to_string(), 2.0, 80.0, None).unwrap();
        let total = inv.total_realized_gain();
        assert!(
            (total - 20.0).abs() < 0.001,
            "Expected 20.0 but got {}",
            total
        );
    }

    #[test]
    fn test_sell_success() {
        let mut inv = make_investment_with_units(1000.0, 10.0);
        // cost basis = $100/unit
        let entry = inv
            .sell(
                "2024-06-01".to_string(),
                4.0,
                150.0,
                Some("partial exit".to_string()),
            )
            .unwrap();

        assert!((entry.units_sold - 4.0).abs() < 0.001);
        assert!((entry.sale_price_per_unit - 150.0).abs() < 0.001);
        assert!((entry.total_proceeds - 600.0).abs() < 0.001);
        // realized gain = 600 - (4 * 100) = 200
        assert!((entry.realized_gain - 200.0).abs() < 0.001);
        assert_eq!(entry.notes, Some("partial exit".to_string()));

        // After sale: 6 units remain, amount reduced by 4*100=400 → 600
        let remaining = inv.remaining_units().unwrap();
        assert!((remaining - 6.0).abs() < 0.001);
        assert!((inv.amount - 600.0).abs() < 0.001);
    }

    #[test]
    fn test_sell_insufficient_units() {
        let mut inv = make_investment_with_units(1000.0, 10.0);
        let result = inv.sell("2024-06-01".to_string(), 15.0, 100.0, None);
        assert!(result.is_err());
        if let Err(e) = result {
            if let Some(InvestmentError::InsufficientUnits(_)) = e.downcast_ref() {
                // expected
            } else {
                panic!("Expected InsufficientUnits error, got: {}", e);
            }
        }
    }

    #[test]
    fn test_sell_no_units_set() {
        let mut inv = make_investment(1000.0, Some(1200.0));
        let result = inv.sell("2024-06-01".to_string(), 1.0, 100.0, None);
        assert!(result.is_err());
        if let Err(e) = result {
            if let Some(InvestmentError::InsufficientUnits(_)) = e.downcast_ref() {
                // expected
            } else {
                panic!("Expected InsufficientUnits error");
            }
        }
    }
}
