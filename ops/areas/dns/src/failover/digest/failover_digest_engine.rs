use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the failover digest calculation.
pub trait FailoverDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFailoverDigestAdjuster {
    pub minimum: f64,
}

impl FailoverDigestModifier for MinimumFailoverDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FailoverDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceFailoverDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverDigestModifier for NamespaceFailoverDigestAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &FailoverDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFailoverDigestAdjuster {
    pub cap: f64,
}

impl FailoverDigestModifier for CapFailoverDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FailoverDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverDigestBurstOffsetAdjuster {
    pub weight: f64,
}

impl FailoverDigestModifier for FailoverDigestBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &FailoverDigest, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

/// Runs the failover digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverDigestEngine {
    pub steps: Vec<Rc<dyn FailoverDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverDigestEngine {
    pub fn compute(&mut self, item: &FailoverDigest) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FailoverDigestModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FailoverDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_failover_digest_engine() -> FailoverDigestEngine {
    FailoverDigestEngine {
        steps: vec![
            Rc::new(MinimumFailoverDigestAdjuster { minimum: 22.9 }) as Rc<dyn FailoverDigestModifier>,
            Rc::new(NamespaceFailoverDigestAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.15)]),
            }),
            Rc::new(CapFailoverDigestAdjuster { cap: 47918.0 }),
            Rc::new(FailoverDigestBurstOffsetAdjuster { weight: 0.79 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 2369.5,
    }
}
