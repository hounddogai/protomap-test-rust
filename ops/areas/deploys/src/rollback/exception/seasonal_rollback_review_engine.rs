use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal rollback review calculation.
pub trait SeasonalRollbackReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalRollbackReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalRollbackReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalRollbackReviewFactor for TieredSeasonalRollbackReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalRollbackReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalRollbackReviewAdjuster {
    pub minimum: f64,
}

impl SeasonalRollbackReviewFactor for MinimumSeasonalRollbackReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalRollbackReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalRollbackReviewAdjuster {
    pub step: f64,
}

impl SeasonalRollbackReviewFactor for RoundingSeasonalRollbackReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalRollbackReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalRollbackReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalRollbackReviewFactor for FlaggedSeasonalRollbackReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalRollbackReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalRollbackReviewAdjuster {
    pub percent: f64,
}

impl SeasonalRollbackReviewFactor for PercentSeasonalRollbackReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalRollbackReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal rollback review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalRollbackReviewEngine {
    pub steps: Vec<Rc<dyn SeasonalRollbackReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalRollbackReviewEngine {
    pub fn price(&mut self, item: &SeasonalRollbackReview) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalRollbackReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_rollback_review_engine(flags: &flags::Registry) -> SeasonalRollbackReviewEngine {
    SeasonalRollbackReviewEngine {
        steps: vec![
            Rc::new(TieredSeasonalRollbackReviewAdjuster {
                bounds: vec![1298.0, 4329.0, 6268.0],
                rates: vec![1.19, 0.93, 1.15],
            }) as Rc<dyn SeasonalRollbackReviewFactor>,
            Rc::new(MinimumSeasonalRollbackReviewAdjuster { minimum: 34.4 }),
            Rc::new(RoundingSeasonalRollbackReviewAdjuster { step: 0.5 }),
            Rc::new(FlaggedSeasonalRollbackReviewAdjuster {
                flags: flags.clone(),
                flag: "seasonal-rollback-review-strict".to_string(),
                percent: 0.1,
            }),
            Rc::new(PercentSeasonalRollbackReviewAdjuster { percent: -3.6 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 91710.1,
    }
}
