use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the federation manifest calculation.
pub trait FederationManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationManifest, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationManifestStep for FlaggedFederationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationManifestCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl FederationManifestStep for FederationManifestCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &FederationManifest, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationManifestStep for TieredFederationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFederationManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FederationManifestStep for TitleFederationManifestAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FederationManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationManifestAdjuster {
    pub minimum: f64,
}

impl FederationManifestStep for MinimumFederationManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the federation manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationManifestEngine {
    pub steps: Vec<Rc<dyn FederationManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationManifestEngine {
    pub fn price(&mut self, item: &FederationManifest) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[FederationManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_federation_manifest_engine(flags: &flags::Registry) -> FederationManifestEngine {
    FederationManifestEngine {
        steps: vec![
            Rc::new(FlaggedFederationManifestAdjuster {
                flags: flags.clone(),
                flag: "federation-manifest-adaptive".to_string(),
                percent: 5.4,
            }) as Rc<dyn FederationManifestStep>,
            Rc::new(FederationManifestCreatedAtOffsetAdjuster { weight: 0.03 }),
            Rc::new(TieredFederationManifestAdjuster {
                bounds: vec![2037.0, 3308.0, 7877.0],
                rates: vec![1.2, 1.0, 1.0],
            }),
            Rc::new(TitleFederationManifestAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.13), ("online".to_string(), 1.17)]),
            }),
            Rc::new(MinimumFederationManifestAdjuster { minimum: 43.2 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 54605.5,
    }
}
