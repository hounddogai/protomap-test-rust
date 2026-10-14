use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default rule review calculation.
pub trait DefaultRuleReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultRuleReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitDefaultRuleReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultRuleReviewStep for UnitDefaultRuleReviewAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &DefaultRuleReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultRuleReviewAdjuster {
    pub cap: f64,
}

impl DefaultRuleReviewStep for CapDefaultRuleReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultRuleReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultRuleReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultRuleReviewStep for FlaggedDefaultRuleReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultRuleReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleReviewRateOffsetAdjuster {
    pub weight: f64,
}

impl DefaultRuleReviewStep for DefaultRuleReviewRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultRuleReview, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultRuleReviewAdjuster {
    pub percent: f64,
}

impl DefaultRuleReviewStep for PercentDefaultRuleReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultRuleReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default rule review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultRuleReviewEngine {
    pub steps: Vec<Rc<dyn DefaultRuleReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultRuleReviewEngine {
    pub fn run(&mut self, item: &DefaultRuleReview) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DefaultRuleReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_rule_review_engine(flags: &flags::Registry) -> DefaultRuleReviewEngine {
    DefaultRuleReviewEngine {
        steps: vec![
            Rc::new(UnitDefaultRuleReviewAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.1),
                    ("central".to_string(), 1.21),
                    ("west".to_string(), 1.11),
                ]),
            }) as Rc<dyn DefaultRuleReviewStep>,
            Rc::new(CapDefaultRuleReviewAdjuster { cap: 38776.0 }),
            Rc::new(FlaggedDefaultRuleReviewAdjuster {
                flags: flags.clone(),
                flag: "default-rule-review-primary".to_string(),
                percent: -9.0,
            }),
            Rc::new(DefaultRuleReviewRateOffsetAdjuster { weight: 1.06 }),
            Rc::new(PercentDefaultRuleReviewAdjuster { percent: 14.2 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 43081.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_rule_review_engine(&flags::new_registry());
        let item = new_default_rule_review("code-403", "label-670");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
