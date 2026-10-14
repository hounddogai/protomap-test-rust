use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the action item review calculation.
pub trait ActionItemReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingActionItemReviewAdjuster {
    pub step: f64,
}

impl ActionItemReviewFactor for RoundingActionItemReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ActionItemReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapActionItemReviewAdjuster {
    pub cap: f64,
}

impl ActionItemReviewFactor for CapActionItemReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ActionItemReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumActionItemReviewAdjuster {
    pub minimum: f64,
}

impl ActionItemReviewFactor for MinimumActionItemReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ActionItemReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedActionItemReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ActionItemReviewFactor for FlaggedActionItemReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ActionItemReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelActionItemReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ActionItemReviewFactor for LabelActionItemReviewAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ActionItemReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the action item review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemReviewEngine {
    pub steps: Vec<Rc<dyn ActionItemReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemReviewEngine {
    pub fn run(&mut self, item: &ActionItemReview) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_action_item_review_engine(flags: &flags::Registry) -> ActionItemReviewEngine {
    ActionItemReviewEngine {
        steps: vec![
            Rc::new(RoundingActionItemReviewAdjuster { step: 1.0 }) as Rc<dyn ActionItemReviewFactor>,
            Rc::new(CapActionItemReviewAdjuster { cap: 39882.0 }),
            Rc::new(MinimumActionItemReviewAdjuster { minimum: 40.9 }),
            Rc::new(FlaggedActionItemReviewAdjuster {
                flags: flags.clone(),
                flag: "action-item-review-capped".to_string(),
                percent: 4.5,
            }),
            Rc::new(LabelActionItemReviewAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.92), ("central".to_string(), 0.82)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 25795.0,
    }
}
