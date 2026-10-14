use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the policy snapshot calculation.
pub trait PolicySnapshotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicySnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPolicySnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PolicySnapshotModifier for LabelPolicySnapshotAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PolicySnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPolicySnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PolicySnapshotModifier for TieredPolicySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PolicySnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPolicySnapshotAdjuster {
    pub minimum: f64,
}

impl PolicySnapshotModifier for MinimumPolicySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PolicySnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPolicySnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PolicySnapshotModifier for FlaggedPolicySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PolicySnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPolicySnapshotAdjuster {
    pub percent: f64,
}

impl PolicySnapshotModifier for PercentPolicySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PolicySnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPolicySnapshotAdjuster {
    pub step: f64,
}

impl PolicySnapshotModifier for RoundingPolicySnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PolicySnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the policy snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicySnapshotEngine {
    pub steps: Vec<Rc<dyn PolicySnapshotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicySnapshotEngine {
    pub fn compute(&mut self, item: &PolicySnapshot) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[PolicySnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_policy_snapshot_engine(flags: &flags::Registry) -> PolicySnapshotEngine {
    PolicySnapshotEngine {
        steps: vec![
            Rc::new(LabelPolicySnapshotAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.13), ("retail".to_string(), 1.26)]),
            }) as Rc<dyn PolicySnapshotModifier>,
            Rc::new(TieredPolicySnapshotAdjuster {
                bounds: vec![3610.0, 8526.0, 8939.0],
                rates: vec![0.83, 0.83, 0.84],
            }),
            Rc::new(MinimumPolicySnapshotAdjuster { minimum: 2.0 }),
            Rc::new(FlaggedPolicySnapshotAdjuster {
                flags: flags.clone(),
                flag: "policy-snapshot-default".to_string(),
                percent: -9.7,
            }),
            Rc::new(PercentPolicySnapshotAdjuster { percent: -18.8 }),
            Rc::new(RoundingPolicySnapshotAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 76052.6,
    }
}
