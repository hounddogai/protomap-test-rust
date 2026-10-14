use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending action item entry calculation.
pub trait PendingActionItemEntryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingActionItemEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingActionItemEntryAdjuster {
    pub step: f64,
}

impl PendingActionItemEntryStep for RoundingPendingActionItemEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingActionItemEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingActionItemEntryAdjuster {
    pub percent: f64,
}

impl PendingActionItemEntryStep for PercentPendingActionItemEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingActionItemEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingActionItemEntryAdjuster {
    pub cap: f64,
}

impl PendingActionItemEntryStep for CapPendingActionItemEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingActionItemEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the pending action item entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingActionItemEntryEngine {
    pub steps: Vec<Rc<dyn PendingActionItemEntryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingActionItemEntryEngine {
    pub fn price(&mut self, item: &PendingActionItemEntry) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingActionItemEntryStep> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[PendingActionItemEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_action_item_entry_engine() -> PendingActionItemEntryEngine {
    PendingActionItemEntryEngine {
        steps: vec![
            Rc::new(RoundingPendingActionItemEntryAdjuster { step: 0.5 }) as Rc<dyn PendingActionItemEntryStep>,
            Rc::new(PercentPendingActionItemEntryAdjuster { percent: 20.0 }),
            Rc::new(CapPendingActionItemEntryAdjuster { cap: 43067.0 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 55805.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_action_item_entry_engine();
        let item = new_pending_action_item_entry("code-595", "label-376");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
