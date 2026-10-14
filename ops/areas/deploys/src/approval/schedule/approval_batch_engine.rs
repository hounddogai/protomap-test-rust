use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval batch calculation.
pub trait ApprovalBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalBatchFactor for TieredApprovalBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalBatchAdjuster {
    pub cap: f64,
}

impl ApprovalBatchFactor for CapApprovalBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryApprovalBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalBatchFactor for CategoryApprovalBatchAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &ApprovalBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalBatchAdjuster {
    pub percent: f64,
}

impl ApprovalBatchFactor for PercentApprovalBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalBatchAdjuster {
    pub minimum: f64,
}

impl ApprovalBatchFactor for MinimumApprovalBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the approval batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalBatchEngine {
    pub steps: Vec<Rc<dyn ApprovalBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalBatchEngine {
    pub fn compute(&mut self, item: &ApprovalBatch) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_approval_batch_engine() -> ApprovalBatchEngine {
    ApprovalBatchEngine {
        steps: vec![
            Rc::new(TieredApprovalBatchAdjuster {
                bounds: vec![709.0, 7144.0],
                rates: vec![1.02, 0.83],
            }) as Rc<dyn ApprovalBatchFactor>,
            Rc::new(CapApprovalBatchAdjuster { cap: 9880.0 }),
            Rc::new(CategoryApprovalBatchAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.77),
                    ("online".to_string(), 1.32),
                    ("east".to_string(), 0.88),
                ]),
            }),
            Rc::new(PercentApprovalBatchAdjuster { percent: -3.0 }),
            Rc::new(MinimumApprovalBatchAdjuster { minimum: 32.8 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 18390.1,
    }
}
