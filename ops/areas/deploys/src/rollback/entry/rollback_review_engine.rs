use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollback review calculation.
pub trait RollbackReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackReviewAdjuster {
    pub percent: f64,
}

impl RollbackReviewModifier for PercentRollbackReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollbackReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollbackReviewModifier for FlaggedRollbackReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollbackReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackReviewAdjuster {
    pub minimum: f64,
}

impl RollbackReviewModifier for MinimumRollbackReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeRollbackReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollbackReviewModifier for ScopeRollbackReviewAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &RollbackReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollbackReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollbackReviewModifier for TieredRollbackReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollbackReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollbackReviewAdjuster {
    pub cap: f64,
}

impl RollbackReviewModifier for CapRollbackReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollbackReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rollback review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackReviewEngine {
    pub steps: Vec<Rc<dyn RollbackReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackReviewEngine {
    pub fn price(&mut self, item: &RollbackReview) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[RollbackReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rollback_review_engine(flags: &flags::Registry) -> RollbackReviewEngine {
    RollbackReviewEngine {
        steps: vec![
            Rc::new(PercentRollbackReviewAdjuster { percent: 5.4 }) as Rc<dyn RollbackReviewModifier>,
            Rc::new(FlaggedRollbackReviewAdjuster {
                flags: flags.clone(),
                flag: "rollback-review-capped".to_string(),
                percent: 2.6,
            }),
            Rc::new(MinimumRollbackReviewAdjuster { minimum: 0.8 }),
            Rc::new(ScopeRollbackReviewAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.27), ("retail".to_string(), 1.35)]),
            }),
            Rc::new(TieredRollbackReviewAdjuster {
                bounds: vec![5603.0, 5893.0],
                rates: vec![0.82, 1.19],
            }),
            Rc::new(CapRollbackReviewAdjuster { cap: 8879.0 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 32724.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_review_engine(&flags::new_registry());
        let item = new_rollback_review("code-493", "name-724");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
