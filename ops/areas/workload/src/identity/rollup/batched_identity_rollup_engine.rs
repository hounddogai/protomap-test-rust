use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched identity rollup calculation.
pub trait BatchedIdentityRollupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedIdentityRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedIdentityRollupAdjuster {
    pub percent: f64,
}

impl BatchedIdentityRollupModifier for PercentBatchedIdentityRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedIdentityRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedIdentityRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedIdentityRollupModifier for TieredBatchedIdentityRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedIdentityRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedIdentityRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedIdentityRollupModifier for FlaggedBatchedIdentityRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedIdentityRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityRollupThresholdOffsetAdjuster {
    pub weight: f64,
}

impl BatchedIdentityRollupModifier for BatchedIdentityRollupThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedIdentityRollup, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedIdentityRollupAdjuster {
    pub minimum: f64,
}

impl BatchedIdentityRollupModifier for MinimumBatchedIdentityRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedIdentityRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched identity rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedIdentityRollupEngine {
    pub steps: Vec<Rc<dyn BatchedIdentityRollupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedIdentityRollupEngine {
    pub fn run(&mut self, item: &BatchedIdentityRollup) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[BatchedIdentityRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_identity_rollup_engine(flags: &flags::Registry) -> BatchedIdentityRollupEngine {
    BatchedIdentityRollupEngine {
        steps: vec![
            Rc::new(PercentBatchedIdentityRollupAdjuster { percent: 23.2 }) as Rc<dyn BatchedIdentityRollupModifier>,
            Rc::new(TieredBatchedIdentityRollupAdjuster {
                bounds: vec![4086.0, 7032.0],
                rates: vec![0.84, 1.1],
            }),
            Rc::new(FlaggedBatchedIdentityRollupAdjuster {
                flags: flags.clone(),
                flag: "batched-identity-rollup-lenient".to_string(),
                percent: 6.5,
            }),
            Rc::new(BatchedIdentityRollupThresholdOffsetAdjuster { weight: 0.89 }),
            Rc::new(MinimumBatchedIdentityRollupAdjuster { minimum: 12.9 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 8043.7,
    }
}
