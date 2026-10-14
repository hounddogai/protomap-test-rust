use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the regional federation adjustment calculation.
pub trait RegionalFederationAdjustmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalFederationAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalFederationAdjustmentAdjuster {
    pub cap: f64,
}

impl RegionalFederationAdjustmentStep for CapRegionalFederationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalFederationAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalFederationAdjustmentAdjuster {
    pub minimum: f64,
}

impl RegionalFederationAdjustmentStep for MinimumRegionalFederationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalFederationAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationAdjustmentRevisionOffsetAdjuster {
    pub weight: f64,
}

impl RegionalFederationAdjustmentStep for RegionalFederationAdjustmentRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalFederationAdjustment, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRegionalFederationAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalFederationAdjustmentStep for LabelRegionalFederationAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RegionalFederationAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the regional federation adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalFederationAdjustmentEngine {
    pub steps: Vec<Rc<dyn RegionalFederationAdjustmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalFederationAdjustmentEngine {
    pub fn price(&mut self, item: &RegionalFederationAdjustment) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalFederationAdjustmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalFederationAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_federation_adjustment_engine() -> RegionalFederationAdjustmentEngine {
    RegionalFederationAdjustmentEngine {
        steps: vec![
            Rc::new(CapRegionalFederationAdjustmentAdjuster { cap: 7642.0 })
                as Rc<dyn RegionalFederationAdjustmentStep>,
            Rc::new(MinimumRegionalFederationAdjustmentAdjuster { minimum: 43.0 }),
            Rc::new(RegionalFederationAdjustmentRevisionOffsetAdjuster { weight: 1.05 }),
            Rc::new(LabelRegionalFederationAdjustmentAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.92), ("retail".to_string(), 0.91)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 75185.2,
    }
}
