use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the verifier bundle calculation.
pub trait VerifierBundleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentVerifierBundleAdjuster {
    pub percent: f64,
}

impl VerifierBundleFactor for PercentVerifierBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &VerifierBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumVerifierBundleAdjuster {
    pub minimum: f64,
}

impl VerifierBundleFactor for MinimumVerifierBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &VerifierBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierBundleAdjuster {
    pub step: f64,
}

impl VerifierBundleFactor for RoundingVerifierBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl VerifierBundleFactor for TieredVerifierBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &VerifierBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBundleMinimumOffsetAdjuster {
    pub weight: f64,
}

impl VerifierBundleFactor for VerifierBundleMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &VerifierBundle, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the verifier bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierBundleEngine {
    pub steps: Vec<Rc<dyn VerifierBundleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierBundleEngine {
    pub fn run(&mut self, item: &VerifierBundle) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[VerifierBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_verifier_bundle_engine() -> VerifierBundleEngine {
    VerifierBundleEngine {
        steps: vec![
            Rc::new(PercentVerifierBundleAdjuster { percent: 15.6 }) as Rc<dyn VerifierBundleFactor>,
            Rc::new(MinimumVerifierBundleAdjuster { minimum: 48.2 }),
            Rc::new(RoundingVerifierBundleAdjuster { step: 1.0 }),
            Rc::new(TieredVerifierBundleAdjuster {
                bounds: vec![8084.0, 9138.0],
                rates: vec![0.98, 1.12],
            }),
            Rc::new(VerifierBundleMinimumOffsetAdjuster { weight: 0.62 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 23432.5,
    }
}
