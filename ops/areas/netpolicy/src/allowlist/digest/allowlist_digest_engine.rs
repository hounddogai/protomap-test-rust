use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the allowlist digest calculation.
pub trait AllowlistDigestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistDigestQuantityOffsetAdjuster {
    pub weight: f64,
}

impl AllowlistDigestFactor for AllowlistDigestQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &AllowlistDigest, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAllowlistDigestAdjuster {
    pub step: f64,
}

impl AllowlistDigestFactor for RoundingAllowlistDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AllowlistDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAllowlistDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AllowlistDigestFactor for FlaggedAllowlistDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AllowlistDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAllowlistDigestAdjuster {
    pub cap: f64,
}

impl AllowlistDigestFactor for CapAllowlistDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AllowlistDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the allowlist digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistDigestEngine {
    pub steps: Vec<Rc<dyn AllowlistDigestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistDigestEngine {
    pub fn run(&mut self, item: &AllowlistDigest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AllowlistDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_allowlist_digest_engine(flags: &flags::Registry) -> AllowlistDigestEngine {
    AllowlistDigestEngine {
        steps: vec![
            Rc::new(AllowlistDigestQuantityOffsetAdjuster { weight: 1.16 }) as Rc<dyn AllowlistDigestFactor>,
            Rc::new(RoundingAllowlistDigestAdjuster { step: 5.0 }),
            Rc::new(FlaggedAllowlistDigestAdjuster {
                flags: flags.clone(),
                flag: "allowlist-digest-pending".to_string(),
                percent: 2.0,
            }),
            Rc::new(CapAllowlistDigestAdjuster { cap: 48748.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 55441.4,
    }
}
