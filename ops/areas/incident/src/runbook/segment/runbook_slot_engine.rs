use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the runbook slot calculation.
pub trait RunbookSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookSlotAdjuster {
    pub cap: f64,
}

impl RunbookSlotStep for CapRunbookSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookSlotMarginOffsetAdjuster {
    pub weight: f64,
}

impl RunbookSlotStep for RunbookSlotMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &RunbookSlot, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRunbookSlotAdjuster {
    pub step: f64,
}

impl RunbookSlotStep for RoundingRunbookSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RunbookSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookSlotAdjuster {
    pub percent: f64,
}

impl RunbookSlotStep for PercentRunbookSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the runbook slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookSlotEngine {
    pub steps: Vec<Rc<dyn RunbookSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookSlotEngine {
    pub fn run(&mut self, item: &RunbookSlot) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RunbookSlotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RunbookSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_runbook_slot_engine() -> RunbookSlotEngine {
    RunbookSlotEngine {
        steps: vec![
            Rc::new(CapRunbookSlotAdjuster { cap: 20476.0 }) as Rc<dyn RunbookSlotStep>,
            Rc::new(RunbookSlotMarginOffsetAdjuster { weight: 1.04 }),
            Rc::new(RoundingRunbookSlotAdjuster { step: 0.25 }),
            Rc::new(PercentRunbookSlotAdjuster { percent: -7.4 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 67034.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_slot_engine();
        let item = new_runbook_slot("code-139", "label-859");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
