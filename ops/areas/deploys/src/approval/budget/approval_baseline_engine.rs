use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval baseline calculation.
pub trait ApprovalBaselineStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalBaselineAdjuster {
    pub step: f64,
}

impl ApprovalBaselineStep for RoundingApprovalBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryApprovalBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalBaselineStep for CountryApprovalBaselineAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &ApprovalBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalBaselineAdjuster {
    pub percent: f64,
}

impl ApprovalBaselineStep for PercentApprovalBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalBaselineAdjuster {
    pub cap: f64,
}

impl ApprovalBaselineStep for CapApprovalBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalBaselineAdjuster {
    pub minimum: f64,
}

impl ApprovalBaselineStep for MinimumApprovalBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBaselineMinimumOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalBaselineStep for ApprovalBaselineMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalBaseline, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the approval baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalBaselineEngine {
    pub steps: Vec<Rc<dyn ApprovalBaselineStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalBaselineEngine {
    pub fn price(&mut self, item: &ApprovalBaseline) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ApprovalBaselineStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_approval_baseline_engine() -> ApprovalBaselineEngine {
    ApprovalBaselineEngine {
        steps: vec![
            Rc::new(RoundingApprovalBaselineAdjuster { step: 5.0 }) as Rc<dyn ApprovalBaselineStep>,
            Rc::new(CountryApprovalBaselineAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.97),
                    ("retail".to_string(), 1.09),
                    ("west".to_string(), 1.3),
                ]),
            }),
            Rc::new(PercentApprovalBaselineAdjuster { percent: -9.4 }),
            Rc::new(CapApprovalBaselineAdjuster { cap: 33241.0 }),
            Rc::new(MinimumApprovalBaselineAdjuster { minimum: 0.9 }),
            Rc::new(ApprovalBaselineMinimumOffsetAdjuster { weight: 0.58 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 41335.6,
    }
}
