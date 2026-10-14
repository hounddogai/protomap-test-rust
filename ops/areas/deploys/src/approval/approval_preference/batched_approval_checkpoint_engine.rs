use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched approval checkpoint calculation.
pub trait BatchedApprovalCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedApprovalCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryBatchedApprovalCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedApprovalCheckpointModifier for CountryBatchedApprovalCheckpointAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &BatchedApprovalCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedApprovalCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedApprovalCheckpointModifier for TieredBatchedApprovalCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedApprovalCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedApprovalCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedApprovalCheckpointModifier for FlaggedBatchedApprovalCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedApprovalCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedApprovalCheckpointCapacityOffsetAdjuster {
    pub weight: f64,
}

impl BatchedApprovalCheckpointModifier for BatchedApprovalCheckpointCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedApprovalCheckpoint, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedApprovalCheckpointAdjuster {
    pub percent: f64,
}

impl BatchedApprovalCheckpointModifier for PercentBatchedApprovalCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedApprovalCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedApprovalCheckpointAdjuster {
    pub minimum: f64,
}

impl BatchedApprovalCheckpointModifier for MinimumBatchedApprovalCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedApprovalCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched approval checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedApprovalCheckpointEngine {
    pub steps: Vec<Rc<dyn BatchedApprovalCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedApprovalCheckpointEngine {
    pub fn compute(&mut self, item: &BatchedApprovalCheckpoint) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[BatchedApprovalCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_approval_checkpoint_engine(flags: &flags::Registry) -> BatchedApprovalCheckpointEngine {
    BatchedApprovalCheckpointEngine {
        steps: vec![
            Rc::new(CountryBatchedApprovalCheckpointAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.95), ("south".to_string(), 0.89)]),
            }) as Rc<dyn BatchedApprovalCheckpointModifier>,
            Rc::new(TieredBatchedApprovalCheckpointAdjuster {
                bounds: vec![1643.0, 1996.0, 9796.0],
                rates: vec![0.82, 1.06, 0.99],
            }),
            Rc::new(FlaggedBatchedApprovalCheckpointAdjuster {
                flags: flags.clone(),
                flag: "batched-approval-checkpoint-batched".to_string(),
                percent: -3.1,
            }),
            Rc::new(BatchedApprovalCheckpointCapacityOffsetAdjuster { weight: 1.62 }),
            Rc::new(PercentBatchedApprovalCheckpointAdjuster { percent: 21.1 }),
            Rc::new(MinimumBatchedApprovalCheckpointAdjuster { minimum: 21.3 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 14399.6,
    }
}
