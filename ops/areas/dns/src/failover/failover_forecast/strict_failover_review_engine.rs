use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict failover review calculation.
pub trait StrictFailoverReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictFailoverReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictFailoverReviewAdjuster {
    pub step: f64,
}

impl StrictFailoverReviewStep for RoundingStrictFailoverReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictFailoverReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictFailoverReviewAdjuster {
    pub minimum: f64,
}

impl StrictFailoverReviewStep for MinimumStrictFailoverReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictFailoverReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceStrictFailoverReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictFailoverReviewStep for NamespaceStrictFailoverReviewAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &StrictFailoverReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the strict failover review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictFailoverReviewEngine {
    pub steps: Vec<Rc<dyn StrictFailoverReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictFailoverReviewEngine {
    pub fn run(&mut self, item: &StrictFailoverReview) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictFailoverReviewStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictFailoverReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_failover_review_engine() -> StrictFailoverReviewEngine {
    StrictFailoverReviewEngine {
        steps: vec![
            Rc::new(RoundingStrictFailoverReviewAdjuster { step: 5.0 }) as Rc<dyn StrictFailoverReviewStep>,
            Rc::new(MinimumStrictFailoverReviewAdjuster { minimum: 12.4 }),
            Rc::new(NamespaceStrictFailoverReviewAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.8), ("north".to_string(), 1.07)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 69356.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_failover_review_engine();
        let item = new_strict_failover_review("code-357", "title-756");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
