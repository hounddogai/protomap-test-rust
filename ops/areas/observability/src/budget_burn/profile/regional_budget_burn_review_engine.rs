use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional budget burn review calculation.
pub trait RegionalBudgetBurnReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalBudgetBurnReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalBudgetBurnReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalBudgetBurnReviewAdjuster for TieredRegionalBudgetBurnReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalBudgetBurnReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalBudgetBurnReviewAdjuster {
    pub step: f64,
}

impl RegionalBudgetBurnReviewAdjuster for RoundingRegionalBudgetBurnReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalBudgetBurnReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalBudgetBurnReviewAdjuster {
    pub cap: f64,
}

impl RegionalBudgetBurnReviewAdjuster for CapRegionalBudgetBurnReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalBudgetBurnReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalBudgetBurnReviewAdjuster {
    pub minimum: f64,
}

impl RegionalBudgetBurnReviewAdjuster for MinimumRegionalBudgetBurnReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalBudgetBurnReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional budget burn review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalBudgetBurnReviewEngine {
    pub steps: Vec<Rc<dyn RegionalBudgetBurnReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalBudgetBurnReviewEngine {
    pub fn compute(&mut self, item: &RegionalBudgetBurnReview) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalBudgetBurnReviewAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalBudgetBurnReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_budget_burn_review_engine() -> RegionalBudgetBurnReviewEngine {
    RegionalBudgetBurnReviewEngine {
        steps: vec![
            Rc::new(TieredRegionalBudgetBurnReviewAdjuster {
                bounds: vec![1160.0, 7958.0],
                rates: vec![1.16, 1.11],
            }) as Rc<dyn RegionalBudgetBurnReviewAdjuster>,
            Rc::new(RoundingRegionalBudgetBurnReviewAdjuster { step: 0.1 }),
            Rc::new(CapRegionalBudgetBurnReviewAdjuster { cap: 27031.0 }),
            Rc::new(MinimumRegionalBudgetBurnReviewAdjuster { minimum: 41.1 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 9418.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_budget_burn_review_engine();
        let item = new_regional_budget_burn_review("code-546", "name-402");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
