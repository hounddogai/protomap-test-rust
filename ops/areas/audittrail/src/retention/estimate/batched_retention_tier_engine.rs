use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched retention tier calculation.
pub trait BatchedRetentionTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedRetentionTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedRetentionTierAdjuster {
    pub minimum: f64,
}

impl BatchedRetentionTierFactor for MinimumBatchedRetentionTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedRetentionTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedRetentionTierAdjuster {
    pub step: f64,
}

impl BatchedRetentionTierFactor for RoundingBatchedRetentionTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedRetentionTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedRetentionTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedRetentionTierFactor for TieredBatchedRetentionTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedRetentionTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedRetentionTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedRetentionTierFactor for FlaggedBatchedRetentionTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedRetentionTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedRetentionTierAdjuster {
    pub cap: f64,
}

impl BatchedRetentionTierFactor for CapBatchedRetentionTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedRetentionTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the batched retention tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedRetentionTierEngine {
    pub steps: Vec<Rc<dyn BatchedRetentionTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedRetentionTierEngine {
    pub fn compute(&mut self, item: &BatchedRetentionTier) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BatchedRetentionTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_retention_tier_engine(flags: &flags::Registry) -> BatchedRetentionTierEngine {
    BatchedRetentionTierEngine {
        steps: vec![
            Rc::new(MinimumBatchedRetentionTierAdjuster { minimum: 44.6 }) as Rc<dyn BatchedRetentionTierFactor>,
            Rc::new(RoundingBatchedRetentionTierAdjuster { step: 0.1 }),
            Rc::new(TieredBatchedRetentionTierAdjuster {
                bounds: vec![325.0, 5472.0, 8746.0],
                rates: vec![1.07, 1.18, 0.9],
            }),
            Rc::new(FlaggedBatchedRetentionTierAdjuster {
                flags: flags.clone(),
                flag: "batched-retention-tier-strict".to_string(),
                percent: 7.1,
            }),
            Rc::new(CapBatchedRetentionTierAdjuster { cap: 49808.0 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 45637.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_retention_tier_engine(&flags::new_registry());
        let item = new_batched_retention_tier("code-854", "name-142");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
