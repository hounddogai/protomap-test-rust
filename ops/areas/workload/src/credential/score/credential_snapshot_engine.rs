use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the credential snapshot calculation.
pub trait CredentialSnapshotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CredentialSnapshotAdjuster for TieredCredentialSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CredentialSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSnapshotRevisionOffsetAdjuster {
    pub weight: f64,
}

impl CredentialSnapshotAdjuster for CredentialSnapshotRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &CredentialSnapshot, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCredentialSnapshotAdjuster {
    pub percent: f64,
}

impl CredentialSnapshotAdjuster for PercentCredentialSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CredentialSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCredentialSnapshotAdjuster {
    pub minimum: f64,
}

impl CredentialSnapshotAdjuster for MinimumCredentialSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CredentialSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCredentialSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CredentialSnapshotAdjuster for FlaggedCredentialSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CredentialSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCredentialSnapshotAdjuster {
    pub cap: f64,
}

impl CredentialSnapshotAdjuster for CapCredentialSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CredentialSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the credential snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialSnapshotEngine {
    pub steps: Vec<Rc<dyn CredentialSnapshotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialSnapshotEngine {
    pub fn run(&mut self, item: &CredentialSnapshot) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[CredentialSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_credential_snapshot_engine(flags: &flags::Registry) -> CredentialSnapshotEngine {
    CredentialSnapshotEngine {
        steps: vec![
            Rc::new(TieredCredentialSnapshotAdjuster {
                bounds: vec![1384.0, 2502.0, 4414.0, 5953.0],
                rates: vec![0.81, 1.18, 1.1, 0.97],
            }) as Rc<dyn CredentialSnapshotAdjuster>,
            Rc::new(CredentialSnapshotRevisionOffsetAdjuster { weight: 1.03 }),
            Rc::new(PercentCredentialSnapshotAdjuster { percent: -1.4 }),
            Rc::new(MinimumCredentialSnapshotAdjuster { minimum: 41.8 }),
            Rc::new(FlaggedCredentialSnapshotAdjuster {
                flags: flags.clone(),
                flag: "credential-snapshot-default".to_string(),
                percent: 9.4,
            }),
            Rc::new(CapCredentialSnapshotAdjuster { cap: 9619.0 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 68811.1,
    }
}
