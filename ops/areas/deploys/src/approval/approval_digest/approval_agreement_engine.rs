use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval agreement calculation.
pub trait ApprovalAgreementFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalAgreementAdjuster {
    pub step: f64,
}

impl ApprovalAgreementFactor for RoundingApprovalAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalAgreementAdjuster {
    pub cap: f64,
}

impl ApprovalAgreementFactor for CapApprovalAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalAgreementAdjuster {
    pub minimum: f64,
}

impl ApprovalAgreementFactor for MinimumApprovalAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAgreementBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalAgreementFactor for ApprovalAgreementBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalAgreement, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalAgreementFactor for TieredApprovalAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the approval agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalAgreementEngine {
    pub steps: Vec<Rc<dyn ApprovalAgreementFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalAgreementEngine {
    pub fn price(&mut self, item: &ApprovalAgreement) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ApprovalAgreementFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_approval_agreement_engine() -> ApprovalAgreementEngine {
    ApprovalAgreementEngine {
        steps: vec![
            Rc::new(RoundingApprovalAgreementAdjuster { step: 0.5 }) as Rc<dyn ApprovalAgreementFactor>,
            Rc::new(CapApprovalAgreementAdjuster { cap: 35429.0 }),
            Rc::new(MinimumApprovalAgreementAdjuster { minimum: 10.0 }),
            Rc::new(ApprovalAgreementBatchSizeOffsetAdjuster { weight: 0.99 }),
            Rc::new(TieredApprovalAgreementAdjuster {
                bounds: vec![1401.0, 4234.0],
                rates: vec![1.19, 1.01],
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 85116.2,
    }
}
