use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the delegation review calculation.
pub trait DelegationReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDelegationReviewAdjuster {
    pub step: f64,
}

impl DelegationReviewAdjuster for RoundingDelegationReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DelegationReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationReviewAdjuster {
    pub minimum: f64,
}

impl DelegationReviewAdjuster for MinimumDelegationReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDelegationReviewAdjuster {
    pub percent: f64,
}

impl DelegationReviewAdjuster for PercentDelegationReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DelegationReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDelegationReviewAdjuster {
    pub cap: f64,
}

impl DelegationReviewAdjuster for CapDelegationReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DelegationReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationReviewAdjuster for TieredDelegationReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationReviewAdjuster for FlaggedDelegationReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the delegation review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationReviewEngine {
    pub steps: Vec<Rc<dyn DelegationReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationReviewEngine {
    pub fn compute(&mut self, item: &DelegationReview) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[DelegationReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_delegation_review_engine(flags: &flags::Registry) -> DelegationReviewEngine {
    DelegationReviewEngine {
        steps: vec![
            Rc::new(RoundingDelegationReviewAdjuster { step: 0.5 }) as Rc<dyn DelegationReviewAdjuster>,
            Rc::new(MinimumDelegationReviewAdjuster { minimum: 17.2 }),
            Rc::new(PercentDelegationReviewAdjuster { percent: -11.8 }),
            Rc::new(CapDelegationReviewAdjuster { cap: 29546.0 }),
            Rc::new(TieredDelegationReviewAdjuster {
                bounds: vec![4529.0, 7610.0],
                rates: vec![1.04, 0.81],
            }),
            Rc::new(FlaggedDelegationReviewAdjuster {
                flags: flags.clone(),
                flag: "delegation-review-tiered".to_string(),
                percent: 7.3,
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 66425.4,
    }
}
