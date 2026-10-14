use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary failover review calculation.
pub trait PrimaryFailoverReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryFailoverReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryFailoverReviewAdjuster {
    pub percent: f64,
}

impl PrimaryFailoverReviewStep for PercentPrimaryFailoverReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryFailoverReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryFailoverReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryFailoverReviewStep for TieredPrimaryFailoverReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryFailoverReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPrimaryFailoverReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryFailoverReviewStep for LabelPrimaryFailoverReviewAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PrimaryFailoverReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryFailoverReviewAdjuster {
    pub minimum: f64,
}

impl PrimaryFailoverReviewStep for MinimumPrimaryFailoverReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryFailoverReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryFailoverReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryFailoverReviewStep for FlaggedPrimaryFailoverReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryFailoverReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary failover review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryFailoverReviewEngine {
    pub steps: Vec<Rc<dyn PrimaryFailoverReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryFailoverReviewEngine {
    pub fn price(&mut self, item: &PrimaryFailoverReview) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryFailoverReviewStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryFailoverReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_failover_review_engine(flags: &flags::Registry) -> PrimaryFailoverReviewEngine {
    PrimaryFailoverReviewEngine {
        steps: vec![
            Rc::new(PercentPrimaryFailoverReviewAdjuster { percent: -20.9 }) as Rc<dyn PrimaryFailoverReviewStep>,
            Rc::new(TieredPrimaryFailoverReviewAdjuster {
                bounds: vec![1560.0, 4612.0, 6540.0],
                rates: vec![0.84, 0.9, 0.91],
            }),
            Rc::new(LabelPrimaryFailoverReviewAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.17), ("east".to_string(), 1.4)]),
            }),
            Rc::new(MinimumPrimaryFailoverReviewAdjuster { minimum: 42.8 }),
            Rc::new(FlaggedPrimaryFailoverReviewAdjuster {
                flags: flags.clone(),
                flag: "primary-failover-review-seasonal".to_string(),
                percent: -4.8,
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 96582.6,
    }
}
