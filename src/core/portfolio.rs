use crate::core::models::Investment;
use crate::core::storage::Storage;
use crate::error::Result;
use std::collections::HashMap;

/// Computes portfolio-wide analytics by aggregating all investments from storage.
pub struct PortfolioAnalytics {
    storage: Storage,
}

impl PortfolioAnalytics {
    /// Create a new `PortfolioAnalytics` backed by `storage`.
    pub fn new(storage: Storage) -> Self {
        PortfolioAnalytics { storage }
    }

    /// Load all investments and compute a [`PortfolioSummary`].
    pub fn get_summary(&self) -> Result<PortfolioSummary> {
        let investments = self.storage.get_all_investments()?;

        let total_invested: f64 = investments.iter().map(|inv| inv.amount).sum();
        let total_current_value = investments
            .iter()
            .map(|inv| inv.current_value.unwrap_or(0.0))
            .sum();

        let total_roi = total_current_value - total_invested;
        let total_roi_percentage = if total_invested > 0.0 {
            (total_roi / total_invested) * 100.0
        } else {
            0.0
        };

        let allocation_by_type = self.calculate_allocation_by_type(&investments);
        let count_by_type = self.calculate_count_by_type(&investments);
        let total_dividends: f64 = investments.iter().map(|i| i.total_dividends()).sum();

        Ok(PortfolioSummary {
            total_investments: investments.len(),
            total_invested,
            total_current_value,
            total_roi,
            total_roi_percentage,
            allocation_by_type,
            count_by_type,
            total_dividends,
        })
    }

    /// Return a map of `InvestmentType → total current value` for the given
    /// investments (falls back to `amount` when `current_value` is unset).
    pub fn calculate_allocation_by_type(&self, investments: &[Investment]) -> HashMap<String, f64> {
        let mut allocation = HashMap::new();

        for investment in investments {
            let type_name = format!("{}", investment.investment_type);
            let current_value = investment.current_value.unwrap_or(investment.amount);
            *allocation.entry(type_name).or_insert(0.0) += current_value;
        }

        allocation
    }

    /// Return a map of `InvestmentType → number of investments` for the given
    /// investments.
    pub fn calculate_count_by_type(&self, investments: &[Investment]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();

        for investment in investments {
            let type_name = format!("{}", investment.investment_type);
            *counts.entry(type_name).or_insert(0) += 1;
        }

        counts
    }
}

/// Aggregated statistics across the entire portfolio.
#[derive(Debug)]
pub struct PortfolioSummary {
    /// Total number of investment positions.
    pub total_investments: usize,
    /// Sum of all `Investment::amount` values (original cost basis).
    pub total_invested: f64,
    /// Sum of all `Investment::current_value` values (0 for untracked positions).
    pub total_current_value: f64,
    /// `total_current_value − total_invested`.
    pub total_roi: f64,
    /// `total_roi / total_invested × 100`.
    pub total_roi_percentage: f64,
    /// Current value broken down by investment type.
    pub allocation_by_type: HashMap<String, f64>,
    /// Position count broken down by investment type.
    pub count_by_type: HashMap<String, usize>,
    /// Sum of all dividend payments across every position.
    pub total_dividends: f64,
}
