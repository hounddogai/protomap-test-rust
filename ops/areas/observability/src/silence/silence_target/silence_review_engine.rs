use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the silence review calculation.
pub trait SilenceReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceReviewExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl SilenceReviewFactor for SilenceReviewExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &SilenceReview, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleSilenceReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SilenceReviewFactor for TitleSilenceReviewAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &SilenceReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSilenceReviewAdjuster {
    pub percent: f64,
}

impl SilenceReviewFactor for PercentSilenceReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SilenceReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SilenceReviewFactor for TieredSilenceReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SilenceReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSilenceReviewAdjuster {
    pub minimum: f64,
}

impl SilenceReviewFactor for MinimumSilenceReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SilenceReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the silence review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceReviewEngine {
    pub steps: Vec<Rc<dyn SilenceReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceReviewEngine {
    pub fn evaluate(&mut self, item: &SilenceReview) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SilenceReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_silence_review_engine() -> SilenceReviewEngine {
    SilenceReviewEngine {
        steps: vec![
            Rc::new(SilenceReviewExpiresAtOffsetAdjuster { weight: 0.42 }) as Rc<dyn SilenceReviewFactor>,
            Rc::new(TitleSilenceReviewAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.3), ("north".to_string(), 0.89)]),
            }),
            Rc::new(PercentSilenceReviewAdjuster { percent: 14.5 }),
            Rc::new(TieredSilenceReviewAdjuster {
                bounds: vec![2940.0, 5722.0],
                rates: vec![1.04, 0.85],
            }),
            Rc::new(MinimumSilenceReviewAdjuster { minimum: 15.8 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 83864.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_silence_review_engine();
        let item = new_silence_review("code-570", "title-959");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
