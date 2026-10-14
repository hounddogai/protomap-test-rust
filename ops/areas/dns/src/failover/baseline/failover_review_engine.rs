use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the failover review calculation.
pub trait FailoverReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFailoverReviewAdjuster {
    pub step: f64,
}

impl FailoverReviewStep for RoundingFailoverReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FailoverReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FailoverReviewStep for TieredFailoverReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FailoverReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFailoverReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FailoverReviewStep for FlaggedFailoverReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FailoverReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFailoverReviewAdjuster {
    pub cap: f64,
}

impl FailoverReviewStep for CapFailoverReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FailoverReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFailoverReviewAdjuster {
    pub percent: f64,
}

impl FailoverReviewStep for PercentFailoverReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FailoverReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the failover review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverReviewEngine {
    pub steps: Vec<Rc<dyn FailoverReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverReviewEngine {
    pub fn price(&mut self, item: &FailoverReview) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FailoverReviewStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FailoverReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_failover_review_engine(flags: &flags::Registry) -> FailoverReviewEngine {
    FailoverReviewEngine {
        steps: vec![
            Rc::new(RoundingFailoverReviewAdjuster { step: 0.25 }) as Rc<dyn FailoverReviewStep>,
            Rc::new(TieredFailoverReviewAdjuster {
                bounds: vec![463.0, 1846.0, 4359.0],
                rates: vec![1.01, 0.95, 0.83],
            }),
            Rc::new(FlaggedFailoverReviewAdjuster {
                flags: flags.clone(),
                flag: "failover-review-adaptive".to_string(),
                percent: -8.6,
            }),
            Rc::new(CapFailoverReviewAdjuster { cap: 22916.0 }),
            Rc::new(PercentFailoverReviewAdjuster { percent: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 54171.5,
    }
}
