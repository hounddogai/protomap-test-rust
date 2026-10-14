use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the limit adjustment calculation.
pub trait LimitAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLimitAdjustmentAdjuster {
    pub minimum: f64,
}

impl LimitAdjustmentModifier for MinimumLimitAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LimitAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLimitAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LimitAdjustmentModifier for LabelLimitAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LimitAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLimitAdjustmentAdjuster {
    pub percent: f64,
}

impl LimitAdjustmentModifier for PercentLimitAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LimitAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the limit adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitAdjustmentEngine {
    pub steps: Vec<Rc<dyn LimitAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitAdjustmentEngine {
    pub fn evaluate(&mut self, item: &LimitAdjustment) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LimitAdjustmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LimitAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_limit_adjustment_engine() -> LimitAdjustmentEngine {
    LimitAdjustmentEngine {
        steps: vec![
            Rc::new(MinimumLimitAdjustmentAdjuster { minimum: 12.8 }) as Rc<dyn LimitAdjustmentModifier>,
            Rc::new(LabelLimitAdjustmentAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.1)]),
            }),
            Rc::new(PercentLimitAdjustmentAdjuster { percent: -11.7 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 46734.0,
    }
}
