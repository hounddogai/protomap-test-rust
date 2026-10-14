use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the federation group calculation.
pub trait FederationGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationGroupAdjuster {
    pub minimum: f64,
}

impl FederationGroupAdjuster for MinimumFederationGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationGroupAdjuster for TieredFederationGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationGroupAdjuster for FlaggedFederationGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the federation group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationGroupEngine {
    pub steps: Vec<Rc<dyn FederationGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationGroupEngine {
    pub fn evaluate(&mut self, item: &FederationGroup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FederationGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_federation_group_engine(flags: &flags::Registry) -> FederationGroupEngine {
    FederationGroupEngine {
        steps: vec![
            Rc::new(MinimumFederationGroupAdjuster { minimum: 4.3 }) as Rc<dyn FederationGroupAdjuster>,
            Rc::new(TieredFederationGroupAdjuster {
                bounds: vec![6838.0, 8266.0, 8783.0],
                rates: vec![1.19, 1.03, 1.2],
            }),
            Rc::new(FlaggedFederationGroupAdjuster {
                flags: flags.clone(),
                flag: "federation-group-scheduled".to_string(),
                percent: -1.9,
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 12125.5,
    }
}
