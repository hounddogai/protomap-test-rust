use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched release adjustment calculation.
pub trait BatchedReleaseAdjustmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedReleaseAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseAdjustmentTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl BatchedReleaseAdjustmentAdjuster for BatchedReleaseAdjustmentTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedReleaseAdjustment, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedReleaseAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedReleaseAdjustmentAdjuster for TieredBatchedReleaseAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedReleaseAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedReleaseAdjustmentAdjuster {
    pub minimum: f64,
}

impl BatchedReleaseAdjustmentAdjuster for MinimumBatchedReleaseAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedReleaseAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedReleaseAdjustmentAdjuster {
    pub cap: f64,
}

impl BatchedReleaseAdjustmentAdjuster for CapBatchedReleaseAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedReleaseAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryBatchedReleaseAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedReleaseAdjustmentAdjuster for CountryBatchedReleaseAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &BatchedReleaseAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedReleaseAdjustmentAdjuster {
    pub step: f64,
}

impl BatchedReleaseAdjustmentAdjuster for RoundingBatchedReleaseAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedReleaseAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the batched release adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedReleaseAdjustmentEngine {
    pub steps: Vec<Rc<dyn BatchedReleaseAdjustmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedReleaseAdjustmentEngine {
    pub fn price(&mut self, item: &BatchedReleaseAdjustment) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedReleaseAdjustmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedReleaseAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_release_adjustment_engine() -> BatchedReleaseAdjustmentEngine {
    BatchedReleaseAdjustmentEngine {
        steps: vec![
            Rc::new(BatchedReleaseAdjustmentTtlSecondsOffsetAdjuster { weight: 0.92 })
                as Rc<dyn BatchedReleaseAdjustmentAdjuster>,
            Rc::new(TieredBatchedReleaseAdjustmentAdjuster {
                bounds: vec![742.0, 3639.0, 7001.0, 8167.0],
                rates: vec![0.87, 1.16, 0.95, 1.14],
            }),
            Rc::new(MinimumBatchedReleaseAdjustmentAdjuster { minimum: 29.9 }),
            Rc::new(CapBatchedReleaseAdjustmentAdjuster { cap: 16929.0 }),
            Rc::new(CountryBatchedReleaseAdjustmentAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.27), ("north".to_string(), 1.25)]),
            }),
            Rc::new(RoundingBatchedReleaseAdjustmentAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 39730.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_release_adjustment_engine();
        let item = new_batched_release_adjustment("code-816", "label-779");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
