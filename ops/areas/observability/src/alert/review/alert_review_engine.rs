use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the alert review calculation.
pub trait AlertReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AlertReviewAdjuster for TieredAlertReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AlertReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAlertReviewAdjuster {
    pub minimum: f64,
}

impl AlertReviewAdjuster for MinimumAlertReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AlertReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAlertReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AlertReviewAdjuster for FlaggedAlertReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AlertReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the alert review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertReviewEngine {
    pub steps: Vec<Rc<dyn AlertReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertReviewEngine {
    pub fn compute(&mut self, item: &AlertReview) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[AlertReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_alert_review_engine(flags: &flags::Registry) -> AlertReviewEngine {
    AlertReviewEngine {
        steps: vec![
            Rc::new(TieredAlertReviewAdjuster {
                bounds: vec![1548.0, 7257.0, 9657.0, 9811.0],
                rates: vec![1.07, 1.09, 0.8, 1.05],
            }) as Rc<dyn AlertReviewAdjuster>,
            Rc::new(MinimumAlertReviewAdjuster { minimum: 10.5 }),
            Rc::new(FlaggedAlertReviewAdjuster {
                flags: flags.clone(),
                flag: "alert-review-global".to_string(),
                percent: 1.7,
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 58891.4,
    }
}
