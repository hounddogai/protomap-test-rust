use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual rule review calculation.
pub trait ManualRuleReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualRuleReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualRuleReviewAdjuster {
    pub minimum: f64,
}

impl ManualRuleReviewStep for MinimumManualRuleReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualRuleReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualRuleReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualRuleReviewStep for FlaggedManualRuleReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualRuleReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualRuleReviewAdjuster {
    pub percent: f64,
}

impl ManualRuleReviewStep for PercentManualRuleReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualRuleReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryManualRuleReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualRuleReviewStep for CountryManualRuleReviewAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &ManualRuleReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualRuleReviewAdjuster {
    pub cap: f64,
}

impl ManualRuleReviewStep for CapManualRuleReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualRuleReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the manual rule review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualRuleReviewEngine {
    pub steps: Vec<Rc<dyn ManualRuleReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualRuleReviewEngine {
    pub fn evaluate(&mut self, item: &ManualRuleReview) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ManualRuleReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_rule_review_engine(flags: &flags::Registry) -> ManualRuleReviewEngine {
    ManualRuleReviewEngine {
        steps: vec![
            Rc::new(MinimumManualRuleReviewAdjuster { minimum: 24.1 }) as Rc<dyn ManualRuleReviewStep>,
            Rc::new(FlaggedManualRuleReviewAdjuster {
                flags: flags.clone(),
                flag: "manual-rule-review-weighted".to_string(),
                percent: 0.2,
            }),
            Rc::new(PercentManualRuleReviewAdjuster { percent: 16.2 }),
            Rc::new(CountryManualRuleReviewAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.01),
                    ("north".to_string(), 1.33),
                    ("south".to_string(), 0.96),
                ]),
            }),
            Rc::new(CapManualRuleReviewAdjuster { cap: 43037.0 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 90740.5,
    }
}
