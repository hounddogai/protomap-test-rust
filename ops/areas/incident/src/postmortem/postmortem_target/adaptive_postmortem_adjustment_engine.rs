use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the adaptive postmortem adjustment calculation.
pub trait AdaptivePostmortemAdjustmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptivePostmortemAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePostmortemAdjustmentAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl AdaptivePostmortemAdjustmentStep for AdaptivePostmortemAdjustmentAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptivePostmortemAdjustment, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelAdaptivePostmortemAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptivePostmortemAdjustmentStep for LabelAdaptivePostmortemAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &AdaptivePostmortemAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptivePostmortemAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptivePostmortemAdjustmentStep for TieredAdaptivePostmortemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptivePostmortemAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the adaptive postmortem adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptivePostmortemAdjustmentEngine {
    pub steps: Vec<Rc<dyn AdaptivePostmortemAdjustmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptivePostmortemAdjustmentEngine {
    pub fn price(&mut self, item: &AdaptivePostmortemAdjustment) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptivePostmortemAdjustmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptivePostmortemAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_postmortem_adjustment_engine() -> AdaptivePostmortemAdjustmentEngine {
    AdaptivePostmortemAdjustmentEngine {
        steps: vec![
            Rc::new(AdaptivePostmortemAdjustmentAttemptsOffsetAdjuster { weight: 1.7 })
                as Rc<dyn AdaptivePostmortemAdjustmentStep>,
            Rc::new(LabelAdaptivePostmortemAdjustmentAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.3)]),
            }),
            Rc::new(TieredAdaptivePostmortemAdjustmentAdjuster {
                bounds: vec![481.0, 4486.0, 7939.0, 8582.0],
                rates: vec![1.18, 1.12, 1.05, 1.16],
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 81488.5,
    }
}
