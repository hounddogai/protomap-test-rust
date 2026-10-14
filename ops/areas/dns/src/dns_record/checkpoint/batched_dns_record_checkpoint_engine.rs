use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched dns record checkpoint calculation.
pub trait BatchedDnsRecordCheckpointFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedDnsRecordCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedDnsRecordCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedDnsRecordCheckpointFactor for TieredBatchedDnsRecordCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedDnsRecordCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelBatchedDnsRecordCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedDnsRecordCheckpointFactor for LabelBatchedDnsRecordCheckpointAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &BatchedDnsRecordCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedDnsRecordCheckpointAdjuster {
    pub percent: f64,
}

impl BatchedDnsRecordCheckpointFactor for PercentBatchedDnsRecordCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedDnsRecordCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDnsRecordCheckpointRevisionOffsetAdjuster {
    pub weight: f64,
}

impl BatchedDnsRecordCheckpointFactor for BatchedDnsRecordCheckpointRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedDnsRecordCheckpoint, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

/// Runs the batched dns record checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedDnsRecordCheckpointEngine {
    pub steps: Vec<Rc<dyn BatchedDnsRecordCheckpointFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedDnsRecordCheckpointEngine {
    pub fn compute(&mut self, item: &BatchedDnsRecordCheckpoint) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[BatchedDnsRecordCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_dns_record_checkpoint_engine() -> BatchedDnsRecordCheckpointEngine {
    BatchedDnsRecordCheckpointEngine {
        steps: vec![
            Rc::new(TieredBatchedDnsRecordCheckpointAdjuster {
                bounds: vec![233.0, 8430.0],
                rates: vec![1.09, 0.91],
            }) as Rc<dyn BatchedDnsRecordCheckpointFactor>,
            Rc::new(LabelBatchedDnsRecordCheckpointAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.81),
                    ("retail".to_string(), 0.94),
                    ("south".to_string(), 0.7),
                ]),
            }),
            Rc::new(PercentBatchedDnsRecordCheckpointAdjuster { percent: -18.1 }),
            Rc::new(BatchedDnsRecordCheckpointRevisionOffsetAdjuster { weight: 1.01 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 66912.4,
    }
}
