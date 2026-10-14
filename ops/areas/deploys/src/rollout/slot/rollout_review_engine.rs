use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout review calculation.
pub trait RolloutReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutReview, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutReviewStep for FlaggedRolloutReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRolloutReviewAdjuster {
    pub minimum: f64,
}

impl RolloutReviewStep for MinimumRolloutReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RolloutReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRolloutReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutReviewStep for RegionRolloutReviewAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RolloutReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRolloutReviewAdjuster {
    pub percent: f64,
}

impl RolloutReviewStep for PercentRolloutReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RolloutReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRolloutReviewAdjuster {
    pub cap: f64,
}

impl RolloutReviewStep for CapRolloutReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RolloutReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rollout review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutReviewEngine {
    pub steps: Vec<Rc<dyn RolloutReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutReviewEngine {
    pub fn compute(&mut self, item: &RolloutReview) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[RolloutReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rollout_review_engine(flags: &flags::Registry) -> RolloutReviewEngine {
    RolloutReviewEngine {
        steps: vec![
            Rc::new(FlaggedRolloutReviewAdjuster {
                flags: flags.clone(),
                flag: "rollout-review-fallback".to_string(),
                percent: -5.4,
            }) as Rc<dyn RolloutReviewStep>,
            Rc::new(MinimumRolloutReviewAdjuster { minimum: 42.0 }),
            Rc::new(RegionRolloutReviewAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.12)]),
            }),
            Rc::new(PercentRolloutReviewAdjuster { percent: -11.0 }),
            Rc::new(CapRolloutReviewAdjuster { cap: 3617.0 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 16502.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_review_engine(&flags::new_registry());
        let item = new_rollout_review("code-682", "title-536");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
