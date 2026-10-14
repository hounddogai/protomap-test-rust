use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global federation target calculation.
pub trait GlobalFederationTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalFederationTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFederationTargetMaximumOffsetAdjuster {
    pub weight: f64,
}

impl GlobalFederationTargetAdjuster for GlobalFederationTargetMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalFederationTarget, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalFederationTargetAdjuster {
    pub step: f64,
}

impl GlobalFederationTargetAdjuster for RoundingGlobalFederationTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalFederationTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalFederationTargetAdjuster {
    pub cap: f64,
}

impl GlobalFederationTargetAdjuster for CapGlobalFederationTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalFederationTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalFederationTargetAdjuster {
    pub percent: f64,
}

impl GlobalFederationTargetAdjuster for PercentGlobalFederationTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalFederationTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalFederationTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalFederationTargetAdjuster for TieredGlobalFederationTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalFederationTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the global federation target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalFederationTargetEngine {
    pub steps: Vec<Rc<dyn GlobalFederationTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalFederationTargetEngine {
    pub fn price(&mut self, item: &GlobalFederationTarget) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalFederationTargetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalFederationTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_federation_target_engine() -> GlobalFederationTargetEngine {
    GlobalFederationTargetEngine {
        steps: vec![
            Rc::new(GlobalFederationTargetMaximumOffsetAdjuster { weight: 1.3 })
                as Rc<dyn GlobalFederationTargetAdjuster>,
            Rc::new(RoundingGlobalFederationTargetAdjuster { step: 0.25 }),
            Rc::new(CapGlobalFederationTargetAdjuster { cap: 39800.0 }),
            Rc::new(PercentGlobalFederationTargetAdjuster { percent: -11.5 }),
            Rc::new(TieredGlobalFederationTargetAdjuster {
                bounds: vec![3130.0, 3277.0, 3736.0, 9724.0],
                rates: vec![0.83, 0.85, 1.13, 1.07],
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 30571.8,
    }
}
