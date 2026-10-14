use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the identity digest calculation.
pub trait IdentityDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityDigestAdjuster {
    pub cap: f64,
}

impl IdentityDigestAdjuster for CapIdentityDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityDigestBudgetOffsetAdjuster {
    pub weight: f64,
}

impl IdentityDigestAdjuster for IdentityDigestBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &IdentityDigest, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentityDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentityDigestAdjuster for FlaggedIdentityDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentityDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the identity digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityDigestEngine {
    pub steps: Vec<Rc<dyn IdentityDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityDigestEngine {
    pub fn price(&mut self, item: &IdentityDigest) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[IdentityDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_identity_digest_engine(flags: &flags::Registry) -> IdentityDigestEngine {
    IdentityDigestEngine {
        steps: vec![
            Rc::new(CapIdentityDigestAdjuster { cap: 5451.0 }) as Rc<dyn IdentityDigestAdjuster>,
            Rc::new(IdentityDigestBudgetOffsetAdjuster { weight: 1.17 }),
            Rc::new(FlaggedIdentityDigestAdjuster {
                flags: flags.clone(),
                flag: "identity-digest-regional".to_string(),
                percent: -0.3,
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 34648.0,
    }
}
