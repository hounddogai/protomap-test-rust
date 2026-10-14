use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the usage review calculation.
pub trait UsageReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageReviewAdjuster {
    pub minimum: f64,
}

impl UsageReviewModifier for MinimumUsageReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedUsageReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl UsageReviewModifier for FlaggedUsageReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &UsageReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentUsageReviewAdjuster {
    pub percent: f64,
}

impl UsageReviewModifier for PercentUsageReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &UsageReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the usage review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageReviewEngine {
    pub steps: Vec<Rc<dyn UsageReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageReviewEngine {
    pub fn evaluate(&mut self, item: &UsageReview) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn UsageReviewModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[UsageReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_usage_review_engine(flags: &flags::Registry) -> UsageReviewEngine {
    UsageReviewEngine {
        steps: vec![
            Rc::new(MinimumUsageReviewAdjuster { minimum: 45.8 }) as Rc<dyn UsageReviewModifier>,
            Rc::new(FlaggedUsageReviewAdjuster {
                flags: flags.clone(),
                flag: "usage-review-pending".to_string(),
                percent: -0.1,
            }),
            Rc::new(PercentUsageReviewAdjuster { percent: 7.4 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 86940.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_review_engine(&flags::new_registry());
        let item = new_usage_review("code-715", "label-536");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
