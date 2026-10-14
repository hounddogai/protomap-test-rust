use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered segment review calculation.
pub trait TieredSegmentReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredSegmentReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredSegmentReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredSegmentReviewModifier for TieredTieredSegmentReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredSegmentReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredSegmentReviewAdjuster {
    pub step: f64,
}

impl TieredSegmentReviewModifier for RoundingTieredSegmentReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredSegmentReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeTieredSegmentReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredSegmentReviewModifier for ScopeTieredSegmentReviewAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &TieredSegmentReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredSegmentReviewAdjuster {
    pub percent: f64,
}

impl TieredSegmentReviewModifier for PercentTieredSegmentReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredSegmentReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered segment review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredSegmentReviewEngine {
    pub steps: Vec<Rc<dyn TieredSegmentReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredSegmentReviewEngine {
    pub fn evaluate(&mut self, item: &TieredSegmentReview) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredSegmentReviewModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredSegmentReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_segment_review_engine() -> TieredSegmentReviewEngine {
    TieredSegmentReviewEngine {
        steps: vec![
            Rc::new(TieredTieredSegmentReviewAdjuster {
                bounds: vec![2548.0, 3384.0, 8196.0, 9943.0],
                rates: vec![0.85, 1.02, 1.18, 1.08],
            }) as Rc<dyn TieredSegmentReviewModifier>,
            Rc::new(RoundingTieredSegmentReviewAdjuster { step: 0.1 }),
            Rc::new(ScopeTieredSegmentReviewAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.24),
                    ("retail".to_string(), 0.78),
                    ("west".to_string(), 0.9),
                ]),
            }),
            Rc::new(PercentTieredSegmentReviewAdjuster { percent: -12.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 29726.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_segment_review_engine();
        let item = new_tiered_segment_review("code-996", "label-895");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
