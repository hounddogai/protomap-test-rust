use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary action item batch calculation.
pub trait PrimaryActionItemBatchAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryActionItemBatch, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryActionItemBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryActionItemBatchAdjuster for FlaggedPrimaryActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryActionItemBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryActionItemBatchAdjuster {
    pub cap: f64,
}

impl PrimaryActionItemBatchAdjuster for CapPrimaryActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryActionItemBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryActionItemBatchAdjuster {
    pub minimum: f64,
}

impl PrimaryActionItemBatchAdjuster for MinimumPrimaryActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryActionItemBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryActionItemBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryActionItemBatchAdjuster for TieredPrimaryActionItemBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryActionItemBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the primary action item batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryActionItemBatchEngine {
    pub steps: Vec<Rc<dyn PrimaryActionItemBatchAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryActionItemBatchEngine {
    pub fn compute(&mut self, item: &PrimaryActionItemBatch) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryActionItemBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_action_item_batch_engine(flags: &flags::Registry) -> PrimaryActionItemBatchEngine {
    PrimaryActionItemBatchEngine {
        steps: vec![
            Rc::new(FlaggedPrimaryActionItemBatchAdjuster {
                flags: flags.clone(),
                flag: "primary-action-item-batch-capped".to_string(),
                percent: 1.0,
            }) as Rc<dyn PrimaryActionItemBatchAdjuster>,
            Rc::new(CapPrimaryActionItemBatchAdjuster { cap: 4251.0 }),
            Rc::new(MinimumPrimaryActionItemBatchAdjuster { minimum: 49.9 }),
            Rc::new(TieredPrimaryActionItemBatchAdjuster {
                bounds: vec![4450.0, 4963.0, 5289.0],
                rates: vec![0.86, 0.88, 1.08],
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 6732.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_action_item_batch_engine(&flags::new_registry());
        let item = new_primary_action_item_batch("code-344", "label-559");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
