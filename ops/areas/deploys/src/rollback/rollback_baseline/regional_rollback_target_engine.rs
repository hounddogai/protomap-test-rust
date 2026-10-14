use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional rollback target calculation.
pub trait RegionalRollbackTargetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalRollbackTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalRollbackTargetAdjuster {
    pub step: f64,
}

impl RegionalRollbackTargetModifier for RoundingRegionalRollbackTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalRollbackTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeRegionalRollbackTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalRollbackTargetModifier for ScopeRegionalRollbackTargetAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &RegionalRollbackTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalRollbackTargetAdjuster {
    pub percent: f64,
}

impl RegionalRollbackTargetModifier for PercentRegionalRollbackTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalRollbackTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRollbackTargetAmountOffsetAdjuster {
    pub weight: f64,
}

impl RegionalRollbackTargetModifier for RegionalRollbackTargetAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalRollbackTarget, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalRollbackTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalRollbackTargetModifier for TieredRegionalRollbackTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalRollbackTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalRollbackTargetAdjuster {
    pub minimum: f64,
}

impl RegionalRollbackTargetModifier for MinimumRegionalRollbackTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalRollbackTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional rollback target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalRollbackTargetEngine {
    pub steps: Vec<Rc<dyn RegionalRollbackTargetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalRollbackTargetEngine {
    pub fn evaluate(&mut self, item: &RegionalRollbackTarget) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalRollbackTargetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalRollbackTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_rollback_target_engine() -> RegionalRollbackTargetEngine {
    RegionalRollbackTargetEngine {
        steps: vec![
            Rc::new(RoundingRegionalRollbackTargetAdjuster { step: 5.0 }) as Rc<dyn RegionalRollbackTargetModifier>,
            Rc::new(ScopeRegionalRollbackTargetAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.31), ("east".to_string(), 0.7)]),
            }),
            Rc::new(PercentRegionalRollbackTargetAdjuster { percent: -10.1 }),
            Rc::new(RegionalRollbackTargetAmountOffsetAdjuster { weight: 1.57 }),
            Rc::new(TieredRegionalRollbackTargetAdjuster {
                bounds: vec![28.0, 5735.0],
                rates: vec![0.81, 0.94],
            }),
            Rc::new(MinimumRegionalRollbackTargetAdjuster { minimum: 19.8 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 41646.6,
    }
}
