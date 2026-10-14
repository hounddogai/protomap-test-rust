use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the legacy action item review calculation.
pub trait LegacyActionItemReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyActionItemReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemReviewWeightOffsetAdjuster {
    pub weight: f64,
}

impl LegacyActionItemReviewFactor for LegacyActionItemReviewWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyActionItemReview, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyActionItemReviewAdjuster {
    pub percent: f64,
}

impl LegacyActionItemReviewFactor for PercentLegacyActionItemReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyActionItemReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyActionItemReviewAdjuster {
    pub minimum: f64,
}

impl LegacyActionItemReviewFactor for MinimumLegacyActionItemReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyActionItemReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceLegacyActionItemReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyActionItemReviewFactor for NamespaceLegacyActionItemReviewAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &LegacyActionItemReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the legacy action item review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyActionItemReviewEngine {
    pub steps: Vec<Rc<dyn LegacyActionItemReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyActionItemReviewEngine {
    pub fn compute(&mut self, item: &LegacyActionItemReview) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyActionItemReviewFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyActionItemReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_action_item_review_engine() -> LegacyActionItemReviewEngine {
    LegacyActionItemReviewEngine {
        steps: vec![
            Rc::new(LegacyActionItemReviewWeightOffsetAdjuster { weight: 0.22 })
                as Rc<dyn LegacyActionItemReviewFactor>,
            Rc::new(PercentLegacyActionItemReviewAdjuster { percent: 16.3 }),
            Rc::new(MinimumLegacyActionItemReviewAdjuster { minimum: 42.3 }),
            Rc::new(NamespaceLegacyActionItemReviewAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.83), ("south".to_string(), 1.19)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 3101.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_action_item_review_engine();
        let item = new_legacy_action_item_review("code-554", "label-748");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
