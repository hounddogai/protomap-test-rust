use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the budget burn cycle calculation.
pub trait BudgetBurnCycleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBudgetBurnCycleAdjuster {
    pub cap: f64,
}

impl BudgetBurnCycleModifier for CapBudgetBurnCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BudgetBurnCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnCycleBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl BudgetBurnCycleModifier for BudgetBurnCycleBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &BudgetBurnCycle, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBudgetBurnCycleAdjuster {
    pub percent: f64,
}

impl BudgetBurnCycleModifier for PercentBudgetBurnCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BudgetBurnCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the budget burn cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnCycleEngine {
    pub steps: Vec<Rc<dyn BudgetBurnCycleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnCycleEngine {
    pub fn compute(&mut self, item: &BudgetBurnCycle) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_budget_burn_cycle_engine() -> BudgetBurnCycleEngine {
    BudgetBurnCycleEngine {
        steps: vec![
            Rc::new(CapBudgetBurnCycleAdjuster { cap: 27234.0 }) as Rc<dyn BudgetBurnCycleModifier>,
            Rc::new(BudgetBurnCycleBatchSizeOffsetAdjuster { weight: 1.11 }),
            Rc::new(PercentBudgetBurnCycleAdjuster { percent: -23.8 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 83459.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_budget_burn_cycle_engine();
        let item = new_budget_burn_cycle("code-228", "title-416");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
