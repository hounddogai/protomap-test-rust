use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global identity manifest calculation.
pub trait GlobalIdentityManifestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalIdentityManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalIdentityManifestAdjuster {
    pub percent: f64,
}

impl GlobalIdentityManifestModifier for PercentGlobalIdentityManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalIdentityManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityManifestMinimumOffsetAdjuster {
    pub weight: f64,
}

impl GlobalIdentityManifestModifier for GlobalIdentityManifestMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalIdentityManifest, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalIdentityManifestAdjuster {
    pub step: f64,
}

impl GlobalIdentityManifestModifier for RoundingGlobalIdentityManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalIdentityManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalIdentityManifestAdjuster {
    pub minimum: f64,
}

impl GlobalIdentityManifestModifier for MinimumGlobalIdentityManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalIdentityManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalIdentityManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalIdentityManifestModifier for TieredGlobalIdentityManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalIdentityManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the global identity manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalIdentityManifestEngine {
    pub steps: Vec<Rc<dyn GlobalIdentityManifestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalIdentityManifestEngine {
    pub fn price(&mut self, item: &GlobalIdentityManifest) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalIdentityManifestModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalIdentityManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_identity_manifest_engine() -> GlobalIdentityManifestEngine {
    GlobalIdentityManifestEngine {
        steps: vec![
            Rc::new(PercentGlobalIdentityManifestAdjuster { percent: 6.5 }) as Rc<dyn GlobalIdentityManifestModifier>,
            Rc::new(GlobalIdentityManifestMinimumOffsetAdjuster { weight: 1.23 }),
            Rc::new(RoundingGlobalIdentityManifestAdjuster { step: 0.1 }),
            Rc::new(MinimumGlobalIdentityManifestAdjuster { minimum: 46.3 }),
            Rc::new(TieredGlobalIdentityManifestAdjuster {
                bounds: vec![566.0, 6500.0, 7350.0, 9587.0],
                rates: vec![1.07, 1.17, 1.2, 1.1],
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 52776.9,
    }
}
