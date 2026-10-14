use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default delegation manifest calculation.
pub trait DefaultDelegationManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultDelegationManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultDelegationManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultDelegationManifestAdjuster for TieredDefaultDelegationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultDelegationManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultDelegationManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultDelegationManifestAdjuster for FlaggedDefaultDelegationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultDelegationManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultDelegationManifestAdjuster {
    pub percent: f64,
}

impl DefaultDelegationManifestAdjuster for PercentDefaultDelegationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultDelegationManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultDelegationManifestAdjuster {
    pub cap: f64,
}

impl DefaultDelegationManifestAdjuster for CapDefaultDelegationManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultDelegationManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeDefaultDelegationManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultDelegationManifestAdjuster for ScopeDefaultDelegationManifestAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &DefaultDelegationManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDelegationManifestPortOffsetAdjuster {
    pub weight: f64,
}

impl DefaultDelegationManifestAdjuster for DefaultDelegationManifestPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultDelegationManifest, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

/// Runs the default delegation manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultDelegationManifestEngine {
    pub steps: Vec<Rc<dyn DefaultDelegationManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultDelegationManifestEngine {
    pub fn price(&mut self, item: &DefaultDelegationManifest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DefaultDelegationManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_delegation_manifest_engine(flags: &flags::Registry) -> DefaultDelegationManifestEngine {
    DefaultDelegationManifestEngine {
        steps: vec![
            Rc::new(TieredDefaultDelegationManifestAdjuster {
                bounds: vec![2293.0, 2465.0, 3273.0, 5666.0],
                rates: vec![1.12, 1.1, 0.97, 1.04],
            }) as Rc<dyn DefaultDelegationManifestAdjuster>,
            Rc::new(FlaggedDefaultDelegationManifestAdjuster {
                flags: flags.clone(),
                flag: "default-delegation-manifest-pending".to_string(),
                percent: -2.6,
            }),
            Rc::new(PercentDefaultDelegationManifestAdjuster { percent: -1.0 }),
            Rc::new(CapDefaultDelegationManifestAdjuster { cap: 40679.0 }),
            Rc::new(ScopeDefaultDelegationManifestAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.78), ("east".to_string(), 0.96)]),
            }),
            Rc::new(DefaultDelegationManifestPortOffsetAdjuster { weight: 1.66 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 20617.9,
    }
}
