use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending retention budget calculation.
pub trait PendingRetentionBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingRetentionBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionBudgetWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl PendingRetentionBudgetFactor for PendingRetentionBudgetWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &PendingRetentionBudget, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingRetentionBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingRetentionBudgetFactor for FlaggedPendingRetentionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingRetentionBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryPendingRetentionBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingRetentionBudgetFactor for CategoryPendingRetentionBudgetAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &PendingRetentionBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingRetentionBudgetAdjuster {
    pub step: f64,
}

impl PendingRetentionBudgetFactor for RoundingPendingRetentionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingRetentionBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingRetentionBudgetAdjuster {
    pub percent: f64,
}

impl PendingRetentionBudgetFactor for PercentPendingRetentionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingRetentionBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingRetentionBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingRetentionBudgetFactor for TieredPendingRetentionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingRetentionBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending retention budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingRetentionBudgetEngine {
    pub steps: Vec<Rc<dyn PendingRetentionBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingRetentionBudgetEngine {
    pub fn price(&mut self, item: &PendingRetentionBudget) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[PendingRetentionBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_retention_budget_engine(flags: &flags::Registry) -> PendingRetentionBudgetEngine {
    PendingRetentionBudgetEngine {
        steps: vec![
            Rc::new(PendingRetentionBudgetWindowDaysOffsetAdjuster { weight: 1.97 })
                as Rc<dyn PendingRetentionBudgetFactor>,
            Rc::new(FlaggedPendingRetentionBudgetAdjuster {
                flags: flags.clone(),
                flag: "pending-retention-budget-default".to_string(),
                percent: -5.5,
            }),
            Rc::new(CategoryPendingRetentionBudgetAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.18),
                    ("central".to_string(), 1.22),
                    ("retail".to_string(), 0.75),
                ]),
            }),
            Rc::new(RoundingPendingRetentionBudgetAdjuster { step: 0.5 }),
            Rc::new(PercentPendingRetentionBudgetAdjuster { percent: 13.7 }),
            Rc::new(TieredPendingRetentionBudgetAdjuster {
                bounds: vec![487.0, 6390.0, 9566.0, 9802.0],
                rates: vec![0.94, 0.85, 1.01, 1.16],
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 20995.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_retention_budget_engine(&flags::new_registry());
        let item = new_pending_retention_budget("code-142", "label-420");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
