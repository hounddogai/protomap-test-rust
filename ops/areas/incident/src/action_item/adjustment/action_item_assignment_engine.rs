use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the action item assignment calculation.
pub trait ActionItemAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumActionItemAssignmentAdjuster {
    pub minimum: f64,
}

impl ActionItemAssignmentModifier for MinimumActionItemAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ActionItemAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentActionItemAssignmentAdjuster {
    pub percent: f64,
}

impl ActionItemAssignmentModifier for PercentActionItemAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ActionItemAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ActionItemAssignmentModifier for TieredActionItemAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ActionItemAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the action item assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemAssignmentEngine {
    pub steps: Vec<Rc<dyn ActionItemAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemAssignmentEngine {
    pub fn price(&mut self, item: &ActionItemAssignment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ActionItemAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_action_item_assignment_engine() -> ActionItemAssignmentEngine {
    ActionItemAssignmentEngine {
        steps: vec![
            Rc::new(MinimumActionItemAssignmentAdjuster { minimum: 23.9 }) as Rc<dyn ActionItemAssignmentModifier>,
            Rc::new(PercentActionItemAssignmentAdjuster { percent: 14.2 }),
            Rc::new(TieredActionItemAssignmentAdjuster {
                bounds: vec![110.0, 2650.0, 9472.0],
                rates: vec![1.14, 0.94, 1.14],
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 84689.9,
    }
}
