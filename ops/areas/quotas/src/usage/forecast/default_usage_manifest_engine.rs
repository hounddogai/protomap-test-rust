use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default usage manifest calculation.
pub trait DefaultUsageManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultUsageManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultUsageManifestAdjuster {
    pub cap: f64,
}

impl DefaultUsageManifestStep for CapDefaultUsageManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultUsageManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultUsageManifestAdjuster {
    pub minimum: f64,
}

impl DefaultUsageManifestStep for MinimumDefaultUsageManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultUsageManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageManifestScoreOffsetAdjuster {
    pub weight: f64,
}

impl DefaultUsageManifestStep for DefaultUsageManifestScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultUsageManifest, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultUsageManifestAdjuster {
    pub step: f64,
}

impl DefaultUsageManifestStep for RoundingDefaultUsageManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultUsageManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceDefaultUsageManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultUsageManifestStep for SourceDefaultUsageManifestAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &DefaultUsageManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultUsageManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultUsageManifestStep for TieredDefaultUsageManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultUsageManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the default usage manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultUsageManifestEngine {
    pub steps: Vec<Rc<dyn DefaultUsageManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultUsageManifestEngine {
    pub fn price(&mut self, item: &DefaultUsageManifest) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultUsageManifestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultUsageManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_usage_manifest_engine() -> DefaultUsageManifestEngine {
    DefaultUsageManifestEngine {
        steps: vec![
            Rc::new(CapDefaultUsageManifestAdjuster { cap: 33391.0 }) as Rc<dyn DefaultUsageManifestStep>,
            Rc::new(MinimumDefaultUsageManifestAdjuster { minimum: 46.7 }),
            Rc::new(DefaultUsageManifestScoreOffsetAdjuster { weight: 0.51 }),
            Rc::new(RoundingDefaultUsageManifestAdjuster { step: 0.05 }),
            Rc::new(SourceDefaultUsageManifestAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.73), ("retail".to_string(), 1.24)]),
            }),
            Rc::new(TieredDefaultUsageManifestAdjuster {
                bounds: vec![933.0, 3528.0, 6013.0, 9896.0],
                rates: vec![1.12, 1.04, 1.18, 0.94],
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 21469.7,
    }
}
