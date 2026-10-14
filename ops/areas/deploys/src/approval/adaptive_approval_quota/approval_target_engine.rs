use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the approval target calculation.
pub trait ApprovalTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalTarget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedApprovalTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ApprovalTargetAdjuster for FlaggedApprovalTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ApprovalTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalTargetGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalTargetAdjuster for ApprovalTargetGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalTarget, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalTargetAdjuster for TieredApprovalTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalTargetAdjuster {
    pub minimum: f64,
}

impl ApprovalTargetAdjuster for MinimumApprovalTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalTargetAdjuster {
    pub step: f64,
}

impl ApprovalTargetAdjuster for RoundingApprovalTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalTargetAdjuster {
    pub percent: f64,
}

impl ApprovalTargetAdjuster for PercentApprovalTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the approval target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalTargetEngine {
    pub steps: Vec<Rc<dyn ApprovalTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalTargetEngine {
    pub fn evaluate(&mut self, item: &ApprovalTarget) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ApprovalTargetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_approval_target_engine(flags: &flags::Registry) -> ApprovalTargetEngine {
    ApprovalTargetEngine {
        steps: vec![
            Rc::new(FlaggedApprovalTargetAdjuster {
                flags: flags.clone(),
                flag: "approval-target-tiered".to_string(),
                percent: -0.3,
            }) as Rc<dyn ApprovalTargetAdjuster>,
            Rc::new(ApprovalTargetGraceDaysOffsetAdjuster { weight: 1.18 }),
            Rc::new(TieredApprovalTargetAdjuster {
                bounds: vec![5306.0, 5625.0, 7764.0, 8906.0],
                rates: vec![1.08, 0.88, 0.85, 1.15],
            }),
            Rc::new(MinimumApprovalTargetAdjuster { minimum: 1.3 }),
            Rc::new(RoundingApprovalTargetAdjuster { step: 0.5 }),
            Rc::new(PercentApprovalTargetAdjuster { percent: 5.1 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 54986.1,
    }
}
