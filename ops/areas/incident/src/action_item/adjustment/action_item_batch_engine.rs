use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the action item batch calculation.
pub trait ActionItemBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumActionItemBatchAdjuster {
    pub minimum: f64,
}

impl ActionItemBatchFactor for MinimumActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ActionItemBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ActionItemBatchFactor for TieredActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ActionItemBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingActionItemBatchAdjuster {
    pub step: f64,
}

impl ActionItemBatchFactor for RoundingActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ActionItemBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentActionItemBatchAdjuster {
    pub percent: f64,
}

impl ActionItemBatchFactor for PercentActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ActionItemBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapActionItemBatchAdjuster {
    pub cap: f64,
}

impl ActionItemBatchFactor for CapActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ActionItemBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the action item batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemBatchEngine {
    pub steps: Vec<Rc<dyn ActionItemBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemBatchEngine {
    pub fn price(&mut self, item: &ActionItemBatch) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ActionItemBatchFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_action_item_batch_engine() -> ActionItemBatchEngine {
    ActionItemBatchEngine {
        steps: vec![
            Rc::new(MinimumActionItemBatchAdjuster { minimum: 16.0 }) as Rc<dyn ActionItemBatchFactor>,
            Rc::new(TieredActionItemBatchAdjuster {
                bounds: vec![6331.0, 8159.0, 9105.0],
                rates: vec![1.19, 1.0, 0.94],
            }),
            Rc::new(RoundingActionItemBatchAdjuster { step: 0.25 }),
            Rc::new(PercentActionItemBatchAdjuster { percent: -14.9 }),
            Rc::new(CapActionItemBatchAdjuster { cap: 40856.0 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 9226.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_action_item_batch_engine();
        let item = new_action_item_batch("code-564", "name-860");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
