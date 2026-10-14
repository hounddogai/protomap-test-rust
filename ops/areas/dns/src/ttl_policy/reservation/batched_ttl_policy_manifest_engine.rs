use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the batched ttl policy manifest calculation.
pub trait BatchedTtlPolicyManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedTtlPolicyManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTtlPolicyManifestCostOffsetAdjuster {
    pub weight: f64,
}

impl BatchedTtlPolicyManifestStep for BatchedTtlPolicyManifestCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedTtlPolicyManifest, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleBatchedTtlPolicyManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedTtlPolicyManifestStep for LocaleBatchedTtlPolicyManifestAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &BatchedTtlPolicyManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedTtlPolicyManifestAdjuster {
    pub minimum: f64,
}

impl BatchedTtlPolicyManifestStep for MinimumBatchedTtlPolicyManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedTtlPolicyManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched ttl policy manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedTtlPolicyManifestEngine {
    pub steps: Vec<Rc<dyn BatchedTtlPolicyManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedTtlPolicyManifestEngine {
    pub fn compute(&mut self, item: &BatchedTtlPolicyManifest) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedTtlPolicyManifestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedTtlPolicyManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_ttl_policy_manifest_engine() -> BatchedTtlPolicyManifestEngine {
    BatchedTtlPolicyManifestEngine {
        steps: vec![
            Rc::new(BatchedTtlPolicyManifestCostOffsetAdjuster { weight: 1.39 })
                as Rc<dyn BatchedTtlPolicyManifestStep>,
            Rc::new(LocaleBatchedTtlPolicyManifestAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.3)]),
            }),
            Rc::new(MinimumBatchedTtlPolicyManifestAdjuster { minimum: 19.8 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 6963.1,
    }
}
