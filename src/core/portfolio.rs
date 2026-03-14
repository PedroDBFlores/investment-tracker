use crate::core::models::Investment;
use crate::core::storage::Storage;
use crate::error::Result;
use std::collections::HashMap;

pub struct PortfolioAnalytics {
    storage: Storage,
}

impl PortfolioAnalytics {
    pub fn new(storage: Storage) -> Self {
        PortfolioAnalytics { storage }
    }

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

    pub fn calculate_allocation_by_type(&self, investments: &[Investment]) -> HashMap<String, f64> {
        let mut allocation = HashMap::new();

        for investment in investments {
            let type_name = format!("{}", investment.investment_type);
            let current_value = investment.current_value.unwrap_or(investment.amount);
            *allocation.entry(type_name).or_insert(0.0) += current_value;
        }

        allocation
    }

    pub fn calculate_count_by_type(&self, investments: &[Investment]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();

        for investment in investments {
            let type_name = format!("{}", investment.investment_type);
            *counts.entry(type_name).or_insert(0) += 1;
        }

        counts
    }
}

#[derive(Debug)]
pub struct PortfolioSummary {
    pub total_investments: usize,
    pub total_invested: f64,
    pub total_current_value: f64,
    pub total_roi: f64,
    pub total_roi_percentage: f64,
    pub allocation_by_type: HashMap<String, f64>,
    pub count_by_type: HashMap<String, usize>,
    pub total_dividends: f64,
}
