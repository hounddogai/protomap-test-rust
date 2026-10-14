use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the overage slot calculation.
pub trait OverageSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumOverageSlotAdjuster {
    pub minimum: f64,
}

impl OverageSlotStep for MinimumOverageSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &OverageSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageSlotBudgetOffsetAdjuster {
    pub weight: f64,
}

impl OverageSlotStep for OverageSlotBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &OverageSlot, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingOverageSlotAdjuster {
    pub step: f64,
}

impl OverageSlotStep for RoundingOverageSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &OverageSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the overage slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageSlotEngine {
    pub steps: Vec<Rc<dyn OverageSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageSlotEngine {
    pub fn evaluate(&mut self, item: &OverageSlot) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[OverageSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_overage_slot_engine() -> OverageSlotEngine {
    OverageSlotEngine {
        steps: vec![
            Rc::new(MinimumOverageSlotAdjuster { minimum: 17.1 }) as Rc<dyn OverageSlotStep>,
            Rc::new(OverageSlotBudgetOffsetAdjuster { weight: 0.86 }),
            Rc::new(RoundingOverageSlotAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 62730.8,
    }
}
