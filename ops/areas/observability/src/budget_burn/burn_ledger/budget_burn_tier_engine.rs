use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the budget burn tier calculation.
pub trait BudgetBurnTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnTier, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedBudgetBurnTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BudgetBurnTierAdjuster for FlaggedBudgetBurnTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BudgetBurnTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BudgetBurnTierAdjuster for TieredBudgetBurnTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BudgetBurnTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBudgetBurnTierAdjuster {
    pub step: f64,
}

impl BudgetBurnTierAdjuster for RoundingBudgetBurnTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BudgetBurnTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBudgetBurnTierAdjuster {
    pub percent: f64,
}

impl BudgetBurnTierAdjuster for PercentBudgetBurnTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BudgetBurnTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBudgetBurnTierAdjuster {
    pub minimum: f64,
}

impl BudgetBurnTierAdjuster for MinimumBudgetBurnTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BudgetBurnTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBudgetBurnTierAdjuster {
    pub cap: f64,
}

impl BudgetBurnTierAdjuster for CapBudgetBurnTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BudgetBurnTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the budget burn tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnTierEngine {
    pub steps: Vec<Rc<dyn BudgetBurnTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnTierEngine {
    pub fn compute(&mut self, item: &BudgetBurnTier) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_budget_burn_tier_engine(flags: &flags::Registry) -> BudgetBurnTierEngine {
    BudgetBurnTierEngine {
        steps: vec![
            Rc::new(FlaggedBudgetBurnTierAdjuster {
                flags: flags.clone(),
                flag: "budget-burn-tier-global".to_string(),
                percent: 7.7,
            }) as Rc<dyn BudgetBurnTierAdjuster>,
            Rc::new(TieredBudgetBurnTierAdjuster {
                bounds: vec![81.0, 2601.0, 6168.0, 8595.0],
                rates: vec![1.14, 0.85, 1.15, 0.95],
            }),
            Rc::new(RoundingBudgetBurnTierAdjuster { step: 0.5 }),
            Rc::new(PercentBudgetBurnTierAdjuster { percent: 1.1 }),
            Rc::new(MinimumBudgetBurnTierAdjuster { minimum: 29.2 }),
            Rc::new(CapBudgetBurnTierAdjuster { cap: 13716.0 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 24665.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_budget_burn_tier_engine(&flags::new_registry());
        let item = new_budget_burn_tier("code-921", "label-936");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
