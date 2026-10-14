use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the default zone review calculation.
pub trait DefaultZoneReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultZoneReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultZoneReviewAdjuster {
    pub percent: f64,
}

impl DefaultZoneReviewStep for PercentDefaultZoneReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultZoneReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultZoneReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultZoneReviewStep for FlaggedDefaultZoneReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultZoneReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneReviewCostOffsetAdjuster {
    pub weight: f64,
}

impl DefaultZoneReviewStep for DefaultZoneReviewCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultZoneReview, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultZoneReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultZoneReviewStep for TieredDefaultZoneReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultZoneReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultZoneReviewAdjuster {
    pub cap: f64,
}

impl DefaultZoneReviewStep for CapDefaultZoneReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultZoneReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the default zone review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultZoneReviewEngine {
    pub steps: Vec<Rc<dyn DefaultZoneReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultZoneReviewEngine {
    pub fn compute(&mut self, item: &DefaultZoneReview) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultZoneReviewStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultZoneReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_zone_review_engine(flags: &flags::Registry) -> DefaultZoneReviewEngine {
    DefaultZoneReviewEngine {
        steps: vec![
            Rc::new(PercentDefaultZoneReviewAdjuster { percent: -17.3 }) as Rc<dyn DefaultZoneReviewStep>,
            Rc::new(FlaggedDefaultZoneReviewAdjuster {
                flags: flags.clone(),
                flag: "default-zone-review-primary".to_string(),
                percent: 10.0,
            }),
            Rc::new(DefaultZoneReviewCostOffsetAdjuster { weight: 0.15 }),
            Rc::new(TieredDefaultZoneReviewAdjuster {
                bounds: vec![4759.0, 7012.0],
                rates: vec![0.97, 1.0],
            }),
            Rc::new(CapDefaultZoneReviewAdjuster { cap: 49021.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 70725.0,
    }
}
