use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the delegation bundle calculation.
pub trait DelegationBundleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDelegationBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DelegationBundleFactor for LabelDelegationBundleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DelegationBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationBundleFactor for TieredDelegationBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDelegationBundleAdjuster {
    pub percent: f64,
}

impl DelegationBundleFactor for PercentDelegationBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DelegationBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the delegation bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationBundleEngine {
    pub steps: Vec<Rc<dyn DelegationBundleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationBundleEngine {
    pub fn run(&mut self, item: &DelegationBundle) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DelegationBundleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DelegationBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_delegation_bundle_engine() -> DelegationBundleEngine {
    DelegationBundleEngine {
        steps: vec![
            Rc::new(LabelDelegationBundleAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.14),
                    ("north".to_string(), 0.78),
                    ("central".to_string(), 1.26),
                ]),
            }) as Rc<dyn DelegationBundleFactor>,
            Rc::new(TieredDelegationBundleAdjuster {
                bounds: vec![2457.0, 8761.0],
                rates: vec![1.07, 0.83],
            }),
            Rc::new(PercentDelegationBundleAdjuster { percent: 18.0 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 98435.9,
    }
}
