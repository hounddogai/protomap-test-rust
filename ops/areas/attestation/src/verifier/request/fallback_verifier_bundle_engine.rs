use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback verifier bundle calculation.
pub trait FallbackVerifierBundleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackVerifierBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackVerifierBundleAdjuster {
    pub percent: f64,
}

impl FallbackVerifierBundleFactor for PercentFallbackVerifierBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackVerifierBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackVerifierBundleAdjuster {
    pub step: f64,
}

impl FallbackVerifierBundleFactor for RoundingFallbackVerifierBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackVerifierBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierBundleStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl FallbackVerifierBundleFactor for FallbackVerifierBundleStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackVerifierBundle, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackVerifierBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackVerifierBundleFactor for TieredFallbackVerifierBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackVerifierBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackVerifierBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackVerifierBundleFactor for FlaggedFallbackVerifierBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackVerifierBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback verifier bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackVerifierBundleEngine {
    pub steps: Vec<Rc<dyn FallbackVerifierBundleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackVerifierBundleEngine {
    pub fn evaluate(&mut self, item: &FallbackVerifierBundle) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FallbackVerifierBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_verifier_bundle_engine(flags: &flags::Registry) -> FallbackVerifierBundleEngine {
    FallbackVerifierBundleEngine {
        steps: vec![
            Rc::new(PercentFallbackVerifierBundleAdjuster { percent: 16.4 }) as Rc<dyn FallbackVerifierBundleFactor>,
            Rc::new(RoundingFallbackVerifierBundleAdjuster { step: 0.25 }),
            Rc::new(FallbackVerifierBundleStartsAtOffsetAdjuster { weight: 1.08 }),
            Rc::new(TieredFallbackVerifierBundleAdjuster {
                bounds: vec![1920.0, 4921.0],
                rates: vec![1.18, 0.99],
            }),
            Rc::new(FlaggedFallbackVerifierBundleAdjuster {
                flags: flags.clone(),
                flag: "fallback-verifier-bundle-global".to_string(),
                percent: 7.0,
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 57871.5,
    }
}
