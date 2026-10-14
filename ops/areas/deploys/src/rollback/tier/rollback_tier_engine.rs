use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rollback tier calculation.
pub trait RollbackTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackTierAdjuster {
    pub percent: f64,
}

impl RollbackTierAdjuster for PercentRollbackTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollbackTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollbackTierAdjuster for TieredRollbackTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollbackTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackTierAdjuster {
    pub minimum: f64,
}

impl RollbackTierAdjuster for MinimumRollbackTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rollback tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackTierEngine {
    pub steps: Vec<Rc<dyn RollbackTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackTierEngine {
    pub fn compute(&mut self, item: &RollbackTier) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RollbackTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rollback_tier_engine() -> RollbackTierEngine {
    RollbackTierEngine {
        steps: vec![
            Rc::new(PercentRollbackTierAdjuster { percent: 16.9 }) as Rc<dyn RollbackTierAdjuster>,
            Rc::new(TieredRollbackTierAdjuster {
                bounds: vec![3665.0, 4506.0, 5235.0],
                rates: vec![0.85, 1.12, 1.15],
            }),
            Rc::new(MinimumRollbackTierAdjuster { minimum: 20.2 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 42748.1,
    }
}
