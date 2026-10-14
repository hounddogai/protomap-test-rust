use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the action item rollup calculation.
pub trait ActionItemRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingActionItemRollupAdjuster {
    pub step: f64,
}

impl ActionItemRollupFactor for RoundingActionItemRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ActionItemRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentActionItemRollupAdjuster {
    pub percent: f64,
}

impl ActionItemRollupFactor for PercentActionItemRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ActionItemRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumActionItemRollupAdjuster {
    pub minimum: f64,
}

impl ActionItemRollupFactor for MinimumActionItemRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ActionItemRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRollupDiscountOffsetAdjuster {
    pub weight: f64,
}

impl ActionItemRollupFactor for ActionItemRollupDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &ActionItemRollup, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ActionItemRollupFactor for TieredActionItemRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ActionItemRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the action item rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemRollupEngine {
    pub steps: Vec<Rc<dyn ActionItemRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemRollupEngine {
    pub fn run(&mut self, item: &ActionItemRollup) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_action_item_rollup_engine() -> ActionItemRollupEngine {
    ActionItemRollupEngine {
        steps: vec![
            Rc::new(RoundingActionItemRollupAdjuster { step: 1.0 }) as Rc<dyn ActionItemRollupFactor>,
            Rc::new(PercentActionItemRollupAdjuster { percent: 5.2 }),
            Rc::new(MinimumActionItemRollupAdjuster { minimum: 20.6 }),
            Rc::new(ActionItemRollupDiscountOffsetAdjuster { weight: 0.88 }),
            Rc::new(TieredActionItemRollupAdjuster {
                bounds: vec![3298.0, 3981.0],
                rates: vec![1.06, 0.88],
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 53905.7,
    }
}
