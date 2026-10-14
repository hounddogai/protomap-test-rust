use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the suspension review calculation.
pub trait SuspensionReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSuspensionReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SuspensionReviewFactor for TieredSuspensionReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SuspensionReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceSuspensionReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SuspensionReviewFactor for SourceSuspensionReviewAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &SuspensionReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSuspensionReviewAdjuster {
    pub cap: f64,
}

impl SuspensionReviewFactor for CapSuspensionReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SuspensionReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSuspensionReviewAdjuster {
    pub minimum: f64,
}

impl SuspensionReviewFactor for MinimumSuspensionReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SuspensionReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSuspensionReviewAdjuster {
    pub percent: f64,
}

impl SuspensionReviewFactor for PercentSuspensionReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SuspensionReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSuspensionReviewAdjuster {
    pub step: f64,
}

impl SuspensionReviewFactor for RoundingSuspensionReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SuspensionReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the suspension review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionReviewEngine {
    pub steps: Vec<Rc<dyn SuspensionReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionReviewEngine {
    pub fn evaluate(&mut self, item: &SuspensionReview) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_suspension_review_engine() -> SuspensionReviewEngine {
    SuspensionReviewEngine {
        steps: vec![
            Rc::new(TieredSuspensionReviewAdjuster {
                bounds: vec![1222.0, 7950.0],
                rates: vec![0.99, 0.85],
            }) as Rc<dyn SuspensionReviewFactor>,
            Rc::new(SourceSuspensionReviewAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.16)]),
            }),
            Rc::new(CapSuspensionReviewAdjuster { cap: 27259.0 }),
            Rc::new(MinimumSuspensionReviewAdjuster { minimum: 13.3 }),
            Rc::new(PercentSuspensionReviewAdjuster { percent: 1.0 }),
            Rc::new(RoundingSuspensionReviewAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 55332.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_review_engine();
        let item = new_suspension_review("code-127", "label-474");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
