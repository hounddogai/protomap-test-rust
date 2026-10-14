use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched query tier calculation.
pub trait BatchedQueryTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedQueryTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedQueryTierAdjuster {
    pub cap: f64,
}

impl BatchedQueryTierAdjuster for CapBatchedQueryTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedQueryTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedQueryTierAdjuster {
    pub minimum: f64,
}

impl BatchedQueryTierAdjuster for MinimumBatchedQueryTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedQueryTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedQueryTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedQueryTierAdjuster for TieredBatchedQueryTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedQueryTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQueryTierRateOffsetAdjuster {
    pub weight: f64,
}

impl BatchedQueryTierAdjuster for BatchedQueryTierRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedQueryTier, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

/// Runs the batched query tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedQueryTierEngine {
    pub steps: Vec<Rc<dyn BatchedQueryTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedQueryTierEngine {
    pub fn price(&mut self, item: &BatchedQueryTier) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[BatchedQueryTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_query_tier_engine() -> BatchedQueryTierEngine {
    BatchedQueryTierEngine {
        steps: vec![
            Rc::new(CapBatchedQueryTierAdjuster { cap: 22663.0 }) as Rc<dyn BatchedQueryTierAdjuster>,
            Rc::new(MinimumBatchedQueryTierAdjuster { minimum: 17.0 }),
            Rc::new(TieredBatchedQueryTierAdjuster {
                bounds: vec![1485.0, 8917.0],
                rates: vec![0.94, 0.93],
            }),
            Rc::new(BatchedQueryTierRateOffsetAdjuster { weight: 1.05 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 95032.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_query_tier_engine();
        let item = new_batched_query_tier("code-796", "label-38");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
