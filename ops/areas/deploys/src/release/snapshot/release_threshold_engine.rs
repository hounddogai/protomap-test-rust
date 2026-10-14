use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the release threshold calculation.
pub trait ReleaseThresholdAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReleaseThresholdAdjuster {
    pub cap: f64,
}

impl ReleaseThresholdAdjuster for CapReleaseThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReleaseThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeReleaseThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReleaseThresholdAdjuster for ScopeReleaseThresholdAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &ReleaseThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseThresholdRetriesOffsetAdjuster {
    pub weight: f64,
}

impl ReleaseThresholdAdjuster for ReleaseThresholdRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &ReleaseThreshold, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReleaseThresholdAdjuster {
    pub percent: f64,
}

impl ReleaseThresholdAdjuster for PercentReleaseThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReleaseThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the release threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseThresholdEngine {
    pub steps: Vec<Rc<dyn ReleaseThresholdAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseThresholdEngine {
    pub fn price(&mut self, item: &ReleaseThreshold) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseThresholdAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_release_threshold_engine() -> ReleaseThresholdEngine {
    ReleaseThresholdEngine {
        steps: vec![
            Rc::new(CapReleaseThresholdAdjuster { cap: 40344.0 }) as Rc<dyn ReleaseThresholdAdjuster>,
            Rc::new(ScopeReleaseThresholdAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.39),
                    ("west".to_string(), 1.36),
                    ("north".to_string(), 1.32),
                ]),
            }),
            Rc::new(ReleaseThresholdRetriesOffsetAdjuster { weight: 0.26 }),
            Rc::new(PercentReleaseThresholdAdjuster { percent: 22.2 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 85318.9,
    }
}
