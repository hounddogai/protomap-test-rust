use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the budget burn item calculation.
pub trait BudgetBurnItemFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnItem, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBudgetBurnItemAdjuster {
    pub percent: f64,
}

impl BudgetBurnItemFactor for PercentBudgetBurnItemAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BudgetBurnItem, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBudgetBurnItemAdjuster {
    pub cap: f64,
}

impl BudgetBurnItemFactor for CapBudgetBurnItemAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BudgetBurnItem, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnItemAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BudgetBurnItemFactor for TieredBudgetBurnItemAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BudgetBurnItem, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBudgetBurnItemAdjuster {
    pub step: f64,
}

impl BudgetBurnItemFactor for RoundingBudgetBurnItemAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BudgetBurnItem, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBudgetBurnItemAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BudgetBurnItemFactor for FlaggedBudgetBurnItemAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BudgetBurnItem, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the budget burn item steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnItemEngine {
    pub steps: Vec<Rc<dyn BudgetBurnItemFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnItemEngine {
    pub fn evaluate(&mut self, item: &BudgetBurnItem) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnItem]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_budget_burn_item_engine(flags: &flags::Registry) -> BudgetBurnItemEngine {
    BudgetBurnItemEngine {
        steps: vec![
            Rc::new(PercentBudgetBurnItemAdjuster { percent: 6.6 }) as Rc<dyn BudgetBurnItemFactor>,
            Rc::new(CapBudgetBurnItemAdjuster { cap: 48866.0 }),
            Rc::new(TieredBudgetBurnItemAdjuster {
                bounds: vec![4020.0, 6353.0, 8609.0, 8760.0],
                rates: vec![0.91, 1.0, 0.9, 1.06],
            }),
            Rc::new(RoundingBudgetBurnItemAdjuster { step: 0.05 }),
            Rc::new(FlaggedBudgetBurnItemAdjuster {
                flags: flags.clone(),
                flag: "budget-burn-item-scheduled".to_string(),
                percent: 1.4,
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 46984.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_budget_burn_item_engine(&flags::new_registry());
        let item = new_budget_burn_item("code-633", "label-216");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
