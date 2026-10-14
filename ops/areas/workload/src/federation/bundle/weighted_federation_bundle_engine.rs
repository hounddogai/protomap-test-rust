use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the weighted federation bundle calculation.
pub trait WeightedFederationBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedFederationBundle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedFederationBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedFederationBundleStep for FlaggedWeightedFederationBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedFederationBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedFederationBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedFederationBundleStep for TieredWeightedFederationBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedFederationBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedFederationBundleAdjuster {
    pub minimum: f64,
}

impl WeightedFederationBundleStep for MinimumWeightedFederationBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedFederationBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the weighted federation bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedFederationBundleEngine {
    pub steps: Vec<Rc<dyn WeightedFederationBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedFederationBundleEngine {
    pub fn price(&mut self, item: &WeightedFederationBundle) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedFederationBundleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedFederationBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_federation_bundle_engine(flags: &flags::Registry) -> WeightedFederationBundleEngine {
    WeightedFederationBundleEngine {
        steps: vec![
            Rc::new(FlaggedWeightedFederationBundleAdjuster {
                flags: flags.clone(),
                flag: "weighted-federation-bundle-strict".to_string(),
                percent: -7.6,
            }) as Rc<dyn WeightedFederationBundleStep>,
            Rc::new(TieredWeightedFederationBundleAdjuster {
                bounds: vec![7291.0, 9849.0],
                rates: vec![0.82, 0.94],
            }),
            Rc::new(MinimumWeightedFederationBundleAdjuster { minimum: 47.0 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 12193.4,
    }
}
