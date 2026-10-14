use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval group calculation.
pub trait ApprovalGroupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalGroupAdjuster {
    pub cap: f64,
}

impl ApprovalGroupModifier for CapApprovalGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelApprovalGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalGroupModifier for LabelApprovalGroupAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ApprovalGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalGroupAdjuster {
    pub minimum: f64,
}

impl ApprovalGroupModifier for MinimumApprovalGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalGroupModifier for TieredApprovalGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalGroupAdjuster {
    pub step: f64,
}

impl ApprovalGroupModifier for RoundingApprovalGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalGroupAdjuster {
    pub percent: f64,
}

impl ApprovalGroupModifier for PercentApprovalGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the approval group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalGroupEngine {
    pub steps: Vec<Rc<dyn ApprovalGroupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalGroupEngine {
    pub fn run(&mut self, item: &ApprovalGroup) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ApprovalGroupModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_approval_group_engine() -> ApprovalGroupEngine {
    ApprovalGroupEngine {
        steps: vec![
            Rc::new(CapApprovalGroupAdjuster { cap: 23554.0 }) as Rc<dyn ApprovalGroupModifier>,
            Rc::new(LabelApprovalGroupAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.86),
                    ("online".to_string(), 0.87),
                    ("central".to_string(), 1.27),
                ]),
            }),
            Rc::new(MinimumApprovalGroupAdjuster { minimum: 11.6 }),
            Rc::new(TieredApprovalGroupAdjuster {
                bounds: vec![1517.0, 2094.0, 7895.0],
                rates: vec![1.02, 0.98, 0.95],
            }),
            Rc::new(RoundingApprovalGroupAdjuster { step: 0.5 }),
            Rc::new(PercentApprovalGroupAdjuster { percent: -22.1 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 19534.0,
    }
}
