use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual budget burn entry calculation.
pub trait ManualBudgetBurnEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualBudgetBurnEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualBudgetBurnEntryAdjuster {
    pub step: f64,
}

impl ManualBudgetBurnEntryAdjuster for RoundingManualBudgetBurnEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualBudgetBurnEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualBudgetBurnEntryAdjuster {
    pub cap: f64,
}

impl ManualBudgetBurnEntryAdjuster for CapManualBudgetBurnEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualBudgetBurnEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualBudgetBurnEntryAdjuster {
    pub minimum: f64,
}

impl ManualBudgetBurnEntryAdjuster for MinimumManualBudgetBurnEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualBudgetBurnEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the manual budget burn entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualBudgetBurnEntryEngine {
    pub steps: Vec<Rc<dyn ManualBudgetBurnEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualBudgetBurnEntryEngine {
    pub fn run(&mut self, item: &ManualBudgetBurnEntry) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ManualBudgetBurnEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_budget_burn_entry_engine() -> ManualBudgetBurnEntryEngine {
    ManualBudgetBurnEntryEngine {
        steps: vec![
            Rc::new(RoundingManualBudgetBurnEntryAdjuster { step: 5.0 }) as Rc<dyn ManualBudgetBurnEntryAdjuster>,
            Rc::new(CapManualBudgetBurnEntryAdjuster { cap: 43939.0 }),
            Rc::new(MinimumManualBudgetBurnEntryAdjuster { minimum: 41.8 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 43806.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_budget_burn_entry_engine();
        let item = new_manual_budget_burn_entry("code-633", "title-652");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
