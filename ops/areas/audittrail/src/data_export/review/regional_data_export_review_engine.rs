use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the regional data export review calculation.
pub trait RegionalDataExportReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalDataExportReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalDataExportReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalDataExportReviewFactor for TieredRegionalDataExportReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalDataExportReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReviewAmountOffsetAdjuster {
    pub weight: f64,
}

impl RegionalDataExportReviewFactor for RegionalDataExportReviewAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalDataExportReview, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalDataExportReviewAdjuster {
    pub percent: f64,
}

impl RegionalDataExportReviewFactor for PercentRegionalDataExportReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalDataExportReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalDataExportReviewAdjuster {
    pub cap: f64,
}

impl RegionalDataExportReviewFactor for CapRegionalDataExportReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalDataExportReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the regional data export review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalDataExportReviewEngine {
    pub steps: Vec<Rc<dyn RegionalDataExportReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalDataExportReviewEngine {
    pub fn compute(&mut self, item: &RegionalDataExportReview) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalDataExportReviewFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalDataExportReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_data_export_review_engine() -> RegionalDataExportReviewEngine {
    RegionalDataExportReviewEngine {
        steps: vec![
            Rc::new(TieredRegionalDataExportReviewAdjuster {
                bounds: vec![1691.0, 2365.0, 3584.0, 6642.0],
                rates: vec![0.88, 0.92, 1.17, 1.16],
            }) as Rc<dyn RegionalDataExportReviewFactor>,
            Rc::new(RegionalDataExportReviewAmountOffsetAdjuster { weight: 1.36 }),
            Rc::new(PercentRegionalDataExportReviewAdjuster { percent: 5.8 }),
            Rc::new(CapRegionalDataExportReviewAdjuster { cap: 22560.0 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 85437.7,
    }
}
