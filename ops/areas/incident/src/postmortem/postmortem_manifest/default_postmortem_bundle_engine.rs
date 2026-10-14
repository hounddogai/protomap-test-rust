use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default postmortem bundle calculation.
pub trait DefaultPostmortemBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultPostmortemBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultPostmortemBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultPostmortemBundleStep for TieredDefaultPostmortemBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultPostmortemBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultPostmortemBundleAdjuster {
    pub step: f64,
}

impl DefaultPostmortemBundleStep for RoundingDefaultPostmortemBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultPostmortemBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultPostmortemBundleAdjuster {
    pub percent: f64,
}

impl DefaultPostmortemBundleStep for PercentDefaultPostmortemBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultPostmortemBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default postmortem bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultPostmortemBundleEngine {
    pub steps: Vec<Rc<dyn DefaultPostmortemBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultPostmortemBundleEngine {
    pub fn evaluate(&mut self, item: &DefaultPostmortemBundle) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultPostmortemBundleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultPostmortemBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_postmortem_bundle_engine() -> DefaultPostmortemBundleEngine {
    DefaultPostmortemBundleEngine {
        steps: vec![
            Rc::new(TieredDefaultPostmortemBundleAdjuster {
                bounds: vec![5803.0, 8771.0],
                rates: vec![0.82, 0.8],
            }) as Rc<dyn DefaultPostmortemBundleStep>,
            Rc::new(RoundingDefaultPostmortemBundleAdjuster { step: 0.1 }),
            Rc::new(PercentDefaultPostmortemBundleAdjuster { percent: -15.9 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 99082.7,
    }
}
