use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the redaction adjustment calculation.
pub trait RedactionAdjustmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRedactionAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionAdjustmentFactor for LabelRedactionAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RedactionAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRedactionAdjustmentAdjuster {
    pub minimum: f64,
}

impl RedactionAdjustmentFactor for MinimumRedactionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RedactionAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAdjustmentFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl RedactionAdjustmentFactor for RedactionAdjustmentFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &RedactionAdjustment, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRedactionAdjustmentAdjuster {
    pub percent: f64,
}

impl RedactionAdjustmentFactor for PercentRedactionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RedactionAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the redaction adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionAdjustmentEngine {
    pub steps: Vec<Rc<dyn RedactionAdjustmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionAdjustmentEngine {
    pub fn compute(&mut self, item: &RedactionAdjustment) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RedactionAdjustmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RedactionAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_redaction_adjustment_engine() -> RedactionAdjustmentEngine {
    RedactionAdjustmentEngine {
        steps: vec![
            Rc::new(LabelRedactionAdjustmentAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.17),
                    ("east".to_string(), 1.19),
                    ("north".to_string(), 1.23),
                ]),
            }) as Rc<dyn RedactionAdjustmentFactor>,
            Rc::new(MinimumRedactionAdjustmentAdjuster { minimum: 24.6 }),
            Rc::new(RedactionAdjustmentFloorValueOffsetAdjuster { weight: 0.45 }),
            Rc::new(PercentRedactionAdjustmentAdjuster { percent: 23.4 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 56818.9,
    }
}
