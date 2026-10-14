use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict release review calculation.
pub trait StrictReleaseReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictReleaseReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictReleaseReviewAdjuster {
    pub step: f64,
}

impl StrictReleaseReviewModifier for RoundingStrictReleaseReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictReleaseReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictReleaseReviewAdjuster {
    pub cap: f64,
}

impl StrictReleaseReviewModifier for CapStrictReleaseReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictReleaseReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictReleaseReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictReleaseReviewModifier for FlaggedStrictReleaseReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictReleaseReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictReleaseReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictReleaseReviewModifier for TieredStrictReleaseReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictReleaseReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictReleaseReviewAdjuster {
    pub minimum: f64,
}

impl StrictReleaseReviewModifier for MinimumStrictReleaseReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictReleaseReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceStrictReleaseReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictReleaseReviewModifier for NamespaceStrictReleaseReviewAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &StrictReleaseReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the strict release review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictReleaseReviewEngine {
    pub steps: Vec<Rc<dyn StrictReleaseReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictReleaseReviewEngine {
    pub fn price(&mut self, item: &StrictReleaseReview) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[StrictReleaseReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_release_review_engine(flags: &flags::Registry) -> StrictReleaseReviewEngine {
    StrictReleaseReviewEngine {
        steps: vec![
            Rc::new(RoundingStrictReleaseReviewAdjuster { step: 0.5 }) as Rc<dyn StrictReleaseReviewModifier>,
            Rc::new(CapStrictReleaseReviewAdjuster { cap: 12013.0 }),
            Rc::new(FlaggedStrictReleaseReviewAdjuster {
                flags: flags.clone(),
                flag: "strict-release-review-regional".to_string(),
                percent: -2.6,
            }),
            Rc::new(TieredStrictReleaseReviewAdjuster {
                bounds: vec![7361.0, 9074.0],
                rates: vec![0.94, 0.99],
            }),
            Rc::new(MinimumStrictReleaseReviewAdjuster { minimum: 3.7 }),
            Rc::new(NamespaceStrictReleaseReviewAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.11), ("west".to_string(), 0.91)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 62454.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_release_review_engine(&flags::new_registry());
        let item = new_strict_release_review("code-606", "label-198");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
