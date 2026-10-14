use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rule review calculation.
pub trait RuleReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRuleReviewAdjuster {
    pub percent: f64,
}

impl RuleReviewModifier for PercentRuleReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RuleReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleReviewFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl RuleReviewModifier for RuleReviewFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &RuleReview, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRuleReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RuleReviewModifier for NameRuleReviewAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RuleReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRuleReviewAdjuster {
    pub minimum: f64,
}

impl RuleReviewModifier for MinimumRuleReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RuleReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRuleReviewAdjuster {
    pub step: f64,
}

impl RuleReviewModifier for RoundingRuleReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RuleReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rule review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleReviewEngine {
    pub steps: Vec<Rc<dyn RuleReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleReviewEngine {
    pub fn price(&mut self, item: &RuleReview) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RuleReviewModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RuleReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rule_review_engine() -> RuleReviewEngine {
    RuleReviewEngine {
        steps: vec![
            Rc::new(PercentRuleReviewAdjuster { percent: 0.7 }) as Rc<dyn RuleReviewModifier>,
            Rc::new(RuleReviewFloorValueOffsetAdjuster { weight: 0.85 }),
            Rc::new(NameRuleReviewAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.21)]),
            }),
            Rc::new(MinimumRuleReviewAdjuster { minimum: 16.1 }),
            Rc::new(RoundingRuleReviewAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 3257.5,
    }
}
