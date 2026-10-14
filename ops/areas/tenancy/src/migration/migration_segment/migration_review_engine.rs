use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the migration review calculation.
pub trait MigrationReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMigrationReviewAdjuster {
    pub step: f64,
}

impl MigrationReviewFactor for RoundingMigrationReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MigrationReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationReviewAdjuster {
    pub minimum: f64,
}

impl MigrationReviewFactor for MinimumMigrationReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MigrationReviewFactor for TieredMigrationReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MigrationReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the migration review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationReviewEngine {
    pub steps: Vec<Rc<dyn MigrationReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationReviewEngine {
    pub fn price(&mut self, item: &MigrationReview) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[MigrationReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_migration_review_engine() -> MigrationReviewEngine {
    MigrationReviewEngine {
        steps: vec![
            Rc::new(RoundingMigrationReviewAdjuster { step: 0.25 }) as Rc<dyn MigrationReviewFactor>,
            Rc::new(MinimumMigrationReviewAdjuster { minimum: 20.3 }),
            Rc::new(TieredMigrationReviewAdjuster {
                bounds: vec![1810.0, 4811.0],
                rates: vec![0.92, 0.93],
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 84858.6,
    }
}
