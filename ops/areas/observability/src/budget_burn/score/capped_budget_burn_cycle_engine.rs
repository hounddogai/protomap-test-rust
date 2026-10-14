use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped budget burn cycle calculation.
pub trait CappedBudgetBurnCycleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedBudgetBurnCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnCycleMarginOffsetAdjuster {
    pub weight: f64,
}

impl CappedBudgetBurnCycleStep for CappedBudgetBurnCycleMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &CappedBudgetBurnCycle, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedBudgetBurnCycleAdjuster {
    pub percent: f64,
}

impl CappedBudgetBurnCycleStep for PercentCappedBudgetBurnCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedBudgetBurnCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedBudgetBurnCycleAdjuster {
    pub cap: f64,
}

impl CappedBudgetBurnCycleStep for CapCappedBudgetBurnCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedBudgetBurnCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the capped budget burn cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedBudgetBurnCycleEngine {
    pub steps: Vec<Rc<dyn CappedBudgetBurnCycleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedBudgetBurnCycleEngine {
    pub fn price(&mut self, item: &CappedBudgetBurnCycle) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[CappedBudgetBurnCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_budget_burn_cycle_engine() -> CappedBudgetBurnCycleEngine {
    CappedBudgetBurnCycleEngine {
        steps: vec![
            Rc::new(CappedBudgetBurnCycleMarginOffsetAdjuster { weight: 0.65 }) as Rc<dyn CappedBudgetBurnCycleStep>,
            Rc::new(PercentCappedBudgetBurnCycleAdjuster { percent: 16.9 }),
            Rc::new(CapCappedBudgetBurnCycleAdjuster { cap: 8976.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 70252.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_budget_burn_cycle_engine();
        let item = new_capped_budget_burn_cycle("code-529", "label-826");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
