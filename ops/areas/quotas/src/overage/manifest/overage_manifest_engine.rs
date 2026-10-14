use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the overage manifest calculation.
pub trait OverageManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingOverageManifestAdjuster {
    pub step: f64,
}

impl OverageManifestAdjuster for RoundingOverageManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &OverageManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl OverageManifestAdjuster for TieredOverageManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &OverageManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapOverageManifestAdjuster {
    pub cap: f64,
}

impl OverageManifestAdjuster for CapOverageManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &OverageManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageManifestRevisionOffsetAdjuster {
    pub weight: f64,
}

impl OverageManifestAdjuster for OverageManifestRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &OverageManifest, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumOverageManifestAdjuster {
    pub minimum: f64,
}

impl OverageManifestAdjuster for MinimumOverageManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &OverageManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedOverageManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl OverageManifestAdjuster for FlaggedOverageManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &OverageManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the overage manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageManifestEngine {
    pub steps: Vec<Rc<dyn OverageManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageManifestEngine {
    pub fn evaluate(&mut self, item: &OverageManifest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[OverageManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_overage_manifest_engine(flags: &flags::Registry) -> OverageManifestEngine {
    OverageManifestEngine {
        steps: vec![
            Rc::new(RoundingOverageManifestAdjuster { step: 0.1 }) as Rc<dyn OverageManifestAdjuster>,
            Rc::new(TieredOverageManifestAdjuster {
                bounds: vec![685.0, 3652.0, 4258.0, 7607.0],
                rates: vec![0.94, 0.87, 0.81, 0.95],
            }),
            Rc::new(CapOverageManifestAdjuster { cap: 25153.0 }),
            Rc::new(OverageManifestRevisionOffsetAdjuster { weight: 0.52 }),
            Rc::new(MinimumOverageManifestAdjuster { minimum: 39.2 }),
            Rc::new(FlaggedOverageManifestAdjuster {
                flags: flags.clone(),
                flag: "overage-manifest-adaptive".to_string(),
                percent: 0.7,
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 42072.1,
    }
}
