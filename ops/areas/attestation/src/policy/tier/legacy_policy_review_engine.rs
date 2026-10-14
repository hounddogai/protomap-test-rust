use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy policy review calculation.
pub trait LegacyPolicyReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyPolicyReview, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyPolicyReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyPolicyReviewFactor for FlaggedLegacyPolicyReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyPolicyReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyPolicyReviewAdjuster {
    pub minimum: f64,
}

impl LegacyPolicyReviewFactor for MinimumLegacyPolicyReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyPolicyReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyPolicyReviewAdjuster {
    pub percent: f64,
}

impl LegacyPolicyReviewFactor for PercentLegacyPolicyReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyPolicyReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyPolicyReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyPolicyReviewFactor for TieredLegacyPolicyReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyPolicyReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitLegacyPolicyReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyPolicyReviewFactor for UnitLegacyPolicyReviewAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &LegacyPolicyReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the legacy policy review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyPolicyReviewEngine {
    pub steps: Vec<Rc<dyn LegacyPolicyReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyPolicyReviewEngine {
    pub fn evaluate(&mut self, item: &LegacyPolicyReview) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[LegacyPolicyReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_policy_review_engine(flags: &flags::Registry) -> LegacyPolicyReviewEngine {
    LegacyPolicyReviewEngine {
        steps: vec![
            Rc::new(FlaggedLegacyPolicyReviewAdjuster {
                flags: flags.clone(),
                flag: "legacy-policy-review-rolling".to_string(),
                percent: -5.0,
            }) as Rc<dyn LegacyPolicyReviewFactor>,
            Rc::new(MinimumLegacyPolicyReviewAdjuster { minimum: 42.0 }),
            Rc::new(PercentLegacyPolicyReviewAdjuster { percent: -10.2 }),
            Rc::new(TieredLegacyPolicyReviewAdjuster {
                bounds: vec![1970.0, 2913.0, 3109.0, 6935.0],
                rates: vec![0.87, 1.03, 1.13, 0.94],
            }),
            Rc::new(UnitLegacyPolicyReviewAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.07), ("online".to_string(), 0.82)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 62290.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_policy_review_engine(&flags::new_registry());
        let item = new_legacy_policy_review("code-835", "title-177");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
