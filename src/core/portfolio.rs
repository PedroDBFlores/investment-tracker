use crate::core::models::Investment;
use crate::core::storage::{Storage, StorageBackend};
use crate::error::Result;
use std::collections::HashMap;

/// Computes portfolio-wide analytics by aggregating all investments from storage.
pub struct PortfolioAnalytics<S = Storage> {
    storage: S,
}

impl<S: StorageBackend> PortfolioAnalytics<S> {
    /// Create a new `PortfolioAnalytics` backed by `storage`.
    pub fn new(storage: S) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::InvestmentType;
    use tempfile::tempdir;

    fn make_investment(
        name: &str,
        amount: f64,
        current_value: Option<f64>,
        inv_type: InvestmentType,
    ) -> Investment {
        Investment::new(
            String::new(),
            inv_type,
            name.to_string(),
            None,
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

    fn analytics_with_investments(investments: Vec<Investment>) -> PortfolioAnalytics<Storage> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("investments.json");
        // Write directly so the dir doesn't get dropped before analytics runs.
        let data = serde_json::to_string(&investments).unwrap();
        std::fs::write(&path, data).unwrap();
        // Keep the TempDir alive by leaking it (test process exits cleanly anyway).
        std::mem::forget(dir);
        PortfolioAnalytics::new(Storage::new(path))
    }

    #[test]
    fn test_summary_empty_portfolio() {
        let analytics = analytics_with_investments(vec![]);
        let summary = analytics.get_summary().unwrap();
        assert_eq!(summary.total_investments, 0);
        assert_eq!(summary.total_invested, 0.0);
        assert_eq!(summary.total_current_value, 0.0);
        assert_eq!(summary.total_roi, 0.0);
        assert_eq!(summary.total_roi_percentage, 0.0);
        assert!(summary.allocation_by_type.is_empty());
        assert!(summary.count_by_type.is_empty());
        assert_eq!(summary.total_dividends, 0.0);
    }

    #[test]
    fn test_summary_single_investment_with_gain() {
        let inv = make_investment("Apple", 1000.0, Some(1200.0), InvestmentType::Stock);
        let analytics = analytics_with_investments(vec![inv]);
        let summary = analytics.get_summary().unwrap();

        assert_eq!(summary.total_investments, 1);
        assert!((summary.total_invested - 1000.0).abs() < 0.001);
        assert!((summary.total_current_value - 1200.0).abs() < 0.001);
        assert!((summary.total_roi - 200.0).abs() < 0.001);
        assert!((summary.total_roi_percentage - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_summary_single_investment_with_loss() {
        let inv = make_investment("Bond", 1000.0, Some(800.0), InvestmentType::Bond);
        let analytics = analytics_with_investments(vec![inv]);
        let summary = analytics.get_summary().unwrap();

        assert!((summary.total_roi - (-200.0)).abs() < 0.001);
        assert!((summary.total_roi_percentage - (-20.0)).abs() < 0.001);
    }

    #[test]
    fn test_summary_no_current_values_uses_zero() {
        let inv = make_investment("Deposit", 500.0, None, InvestmentType::Deposit);
        let analytics = analytics_with_investments(vec![inv]);
        let summary = analytics.get_summary().unwrap();
        // current_value is None → counts as 0
        assert_eq!(summary.total_current_value, 0.0);
        assert_eq!(summary.total_roi, -500.0);
    }

    #[test]
    fn test_summary_totals_across_multiple_investments() {
        let investments = vec![
            make_investment("A", 1000.0, Some(1100.0), InvestmentType::Stock),
            make_investment("B", 2000.0, Some(2200.0), InvestmentType::ETF),
            make_investment("C", 500.0, None, InvestmentType::Crypto),
        ];
        let analytics = analytics_with_investments(investments);
        let summary = analytics.get_summary().unwrap();

        assert_eq!(summary.total_investments, 3);
        assert!((summary.total_invested - 3500.0).abs() < 0.001);
        assert!((summary.total_current_value - 3300.0).abs() < 0.001);
        assert!((summary.total_roi - (-200.0)).abs() < 0.001);
    }

    #[test]
    fn test_allocation_by_type_sums_current_values() {
        let analytics = PortfolioAnalytics::new(Storage::new("/dev/null".into()));
        let investments = vec![
            make_investment("S1", 100.0, Some(150.0), InvestmentType::Stock),
            make_investment("S2", 200.0, Some(250.0), InvestmentType::Stock),
            make_investment("E1", 300.0, Some(320.0), InvestmentType::ETF),
            make_investment("D1", 400.0, None, InvestmentType::Deposit), // no current_value → uses amount
        ];
        let alloc = analytics.calculate_allocation_by_type(&investments);

        assert!((alloc["Stock"] - 400.0).abs() < 0.001); // 150 + 250
        assert!((alloc["ETF"] - 320.0).abs() < 0.001);
        assert!((alloc["Deposit"] - 400.0).abs() < 0.001); // fallback to amount
    }

    #[test]
    fn test_count_by_type() {
        let analytics = PortfolioAnalytics::new(Storage::new("/dev/null".into()));
        let investments = vec![
            make_investment("S1", 100.0, None, InvestmentType::Stock),
            make_investment("S2", 100.0, None, InvestmentType::Stock),
            make_investment("E1", 100.0, None, InvestmentType::ETF),
        ];
        let counts = analytics.calculate_count_by_type(&investments);

        assert_eq!(counts["Stock"], 2);
        assert_eq!(counts["ETF"], 1);
        assert!(!counts.contains_key("Crypto"));
    }

    #[test]
    fn test_summary_includes_dividends() {
        let mut inv = make_investment("MSFT", 1000.0, Some(1100.0), InvestmentType::Stock);
        inv.add_dividend("2024-06-01".to_string(), 50.0, None).unwrap();
        inv.add_dividend("2024-09-01".to_string(), 50.0, None).unwrap();

        let analytics = analytics_with_investments(vec![inv]);
        let summary = analytics.get_summary().unwrap();
        assert!((summary.total_dividends - 100.0).abs() < 0.001);
    }
}
