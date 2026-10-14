use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the delegation manifest calculation.
pub trait DelegationManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDelegationManifestAdjuster {
    pub percent: f64,
}

impl DelegationManifestFactor for PercentDelegationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DelegationManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDelegationManifestAdjuster {
    pub cap: f64,
}

impl DelegationManifestFactor for CapDelegationManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DelegationManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDelegationManifestAdjuster {
    pub step: f64,
}

impl DelegationManifestFactor for RoundingDelegationManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DelegationManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationManifestFactor for TieredDelegationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleDelegationManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DelegationManifestFactor for LocaleDelegationManifestAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &DelegationManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationManifestFactor for FlaggedDelegationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the delegation manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationManifestEngine {
    pub steps: Vec<Rc<dyn DelegationManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationManifestEngine {
    pub fn price(&mut self, item: &DelegationManifest) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DelegationManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_delegation_manifest_engine(flags: &flags::Registry) -> DelegationManifestEngine {
    DelegationManifestEngine {
        steps: vec![
            Rc::new(PercentDelegationManifestAdjuster { percent: -23.8 }) as Rc<dyn DelegationManifestFactor>,
            Rc::new(CapDelegationManifestAdjuster { cap: 22024.0 }),
            Rc::new(RoundingDelegationManifestAdjuster { step: 0.25 }),
            Rc::new(TieredDelegationManifestAdjuster {
                bounds: vec![18.0, 4105.0, 6237.0, 6868.0],
                rates: vec![0.98, 1.06, 0.89, 0.97],
            }),
            Rc::new(LocaleDelegationManifestAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.16), ("online".to_string(), 1.36)]),
            }),
            Rc::new(FlaggedDelegationManifestAdjuster {
                flags: flags.clone(),
                flag: "delegation-manifest-batched".to_string(),
                percent: -4.8,
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 4781.0,
    }
}
