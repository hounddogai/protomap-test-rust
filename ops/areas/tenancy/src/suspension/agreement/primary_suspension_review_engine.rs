use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the primary suspension review calculation.
pub trait PrimarySuspensionReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimarySuspensionReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimarySuspensionReviewAdjuster {
    pub minimum: f64,
}

impl PrimarySuspensionReviewModifier for MinimumPrimarySuspensionReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimarySuspensionReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimarySuspensionReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimarySuspensionReviewModifier for TieredPrimarySuspensionReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimarySuspensionReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimarySuspensionReviewAdjuster {
    pub cap: f64,
}

impl PrimarySuspensionReviewModifier for CapPrimarySuspensionReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimarySuspensionReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary suspension review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimarySuspensionReviewEngine {
    pub steps: Vec<Rc<dyn PrimarySuspensionReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimarySuspensionReviewEngine {
    pub fn price(&mut self, item: &PrimarySuspensionReview) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimarySuspensionReviewModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimarySuspensionReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_suspension_review_engine() -> PrimarySuspensionReviewEngine {
    PrimarySuspensionReviewEngine {
        steps: vec![
            Rc::new(MinimumPrimarySuspensionReviewAdjuster { minimum: 15.2 })
                as Rc<dyn PrimarySuspensionReviewModifier>,
            Rc::new(TieredPrimarySuspensionReviewAdjuster {
                bounds: vec![41.0, 9757.0],
                rates: vec![0.9, 1.18],
            }),
            Rc::new(CapPrimarySuspensionReviewAdjuster { cap: 7567.0 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 66098.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_suspension_review_engine();
        let item = new_primary_suspension_review("code-844", "label-802");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
