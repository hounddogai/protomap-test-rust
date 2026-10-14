use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the usage digest calculation.
pub trait UsageDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingUsageDigestAdjuster {
    pub step: f64,
}

impl UsageDigestModifier for RoundingUsageDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &UsageDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageDigestAdjuster {
    pub minimum: f64,
}

impl UsageDigestModifier for MinimumUsageDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageDigestStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl UsageDigestModifier for UsageDigestStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &UsageDigest, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapUsageDigestAdjuster {
    pub cap: f64,
}

impl UsageDigestModifier for CapUsageDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &UsageDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeUsageDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsageDigestModifier for ScopeUsageDigestAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &UsageDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the usage digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageDigestEngine {
    pub steps: Vec<Rc<dyn UsageDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageDigestEngine {
    pub fn run(&mut self, item: &UsageDigest) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[UsageDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_usage_digest_engine() -> UsageDigestEngine {
    UsageDigestEngine {
        steps: vec![
            Rc::new(RoundingUsageDigestAdjuster { step: 1.0 }) as Rc<dyn UsageDigestModifier>,
            Rc::new(MinimumUsageDigestAdjuster { minimum: 21.6 }),
            Rc::new(UsageDigestStartsAtOffsetAdjuster { weight: 1.27 }),
            Rc::new(CapUsageDigestAdjuster { cap: 12712.0 }),
            Rc::new(ScopeUsageDigestAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.12), ("west".to_string(), 1.29)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 30619.6,
    }
}
