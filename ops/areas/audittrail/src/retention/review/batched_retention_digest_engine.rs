use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the batched retention digest calculation.
pub trait BatchedRetentionDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedRetentionDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedRetentionDigestAdjuster {
    pub cap: f64,
}

impl BatchedRetentionDigestAdjuster for CapBatchedRetentionDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedRetentionDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionDigestCeilingOffsetAdjuster {
    pub weight: f64,
}

impl BatchedRetentionDigestAdjuster for BatchedRetentionDigestCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedRetentionDigest, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBatchedRetentionDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedRetentionDigestAdjuster for NameBatchedRetentionDigestAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BatchedRetentionDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedRetentionDigestAdjuster {
    pub minimum: f64,
}

impl BatchedRetentionDigestAdjuster for MinimumBatchedRetentionDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedRetentionDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched retention digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedRetentionDigestEngine {
    pub steps: Vec<Rc<dyn BatchedRetentionDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedRetentionDigestEngine {
    pub fn compute(&mut self, item: &BatchedRetentionDigest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedRetentionDigestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedRetentionDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_retention_digest_engine() -> BatchedRetentionDigestEngine {
    BatchedRetentionDigestEngine {
        steps: vec![
            Rc::new(CapBatchedRetentionDigestAdjuster { cap: 17254.0 }) as Rc<dyn BatchedRetentionDigestAdjuster>,
            Rc::new(BatchedRetentionDigestCeilingOffsetAdjuster { weight: 1.12 }),
            Rc::new(NameBatchedRetentionDigestAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.71),
                    ("central".to_string(), 1.27),
                    ("south".to_string(), 1.06),
                ]),
            }),
            Rc::new(MinimumBatchedRetentionDigestAdjuster { minimum: 15.8 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 32500.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_retention_digest_engine();
        let item = new_batched_retention_digest("code-305", "name-931");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
