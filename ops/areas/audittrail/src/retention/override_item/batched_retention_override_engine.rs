use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched retention override calculation.
pub trait BatchedRetentionOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedRetentionOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedRetentionOverrideAdjuster {
    pub minimum: f64,
}

impl BatchedRetentionOverrideModifier for MinimumBatchedRetentionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedRetentionOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedRetentionOverrideAdjuster {
    pub cap: f64,
}

impl BatchedRetentionOverrideModifier for CapBatchedRetentionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedRetentionOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedRetentionOverrideAdjuster {
    pub step: f64,
}

impl BatchedRetentionOverrideModifier for RoundingBatchedRetentionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedRetentionOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedRetentionOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedRetentionOverrideModifier for TieredBatchedRetentionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedRetentionOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionOverrideStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl BatchedRetentionOverrideModifier for BatchedRetentionOverrideStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedRetentionOverride, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

/// Runs the batched retention override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedRetentionOverrideEngine {
    pub steps: Vec<Rc<dyn BatchedRetentionOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedRetentionOverrideEngine {
    pub fn price(&mut self, item: &BatchedRetentionOverride) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[BatchedRetentionOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_retention_override_engine() -> BatchedRetentionOverrideEngine {
    BatchedRetentionOverrideEngine {
        steps: vec![
            Rc::new(MinimumBatchedRetentionOverrideAdjuster { minimum: 14.5 })
                as Rc<dyn BatchedRetentionOverrideModifier>,
            Rc::new(CapBatchedRetentionOverrideAdjuster { cap: 45359.0 }),
            Rc::new(RoundingBatchedRetentionOverrideAdjuster { step: 0.1 }),
            Rc::new(TieredBatchedRetentionOverrideAdjuster {
                bounds: vec![3490.0, 4256.0],
                rates: vec![0.85, 1.2],
            }),
            Rc::new(BatchedRetentionOverrideStartsAtOffsetAdjuster { weight: 1.62 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 69527.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_retention_override_engine();
        let item = new_batched_retention_override("code-193", "title-178");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
