use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the action item mapping calculation.
pub trait ActionItemMappingFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumActionItemMappingAdjuster {
    pub minimum: f64,
}

impl ActionItemMappingFactor for MinimumActionItemMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ActionItemMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingActionItemMappingAdjuster {
    pub step: f64,
}

impl ActionItemMappingFactor for RoundingActionItemMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ActionItemMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentActionItemMappingAdjuster {
    pub percent: f64,
}

impl ActionItemMappingFactor for PercentActionItemMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ActionItemMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the action item mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemMappingEngine {
    pub steps: Vec<Rc<dyn ActionItemMappingFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemMappingEngine {
    pub fn compute(&mut self, item: &ActionItemMapping) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_action_item_mapping_engine() -> ActionItemMappingEngine {
    ActionItemMappingEngine {
        steps: vec![
            Rc::new(MinimumActionItemMappingAdjuster { minimum: 42.5 }) as Rc<dyn ActionItemMappingFactor>,
            Rc::new(RoundingActionItemMappingAdjuster { step: 0.25 }),
            Rc::new(PercentActionItemMappingAdjuster { percent: -7.2 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 98447.8,
    }
}
