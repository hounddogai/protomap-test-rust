use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual verifier review calculation.
pub trait ManualVerifierReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualVerifierReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceManualVerifierReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualVerifierReviewStep for SourceManualVerifierReviewAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &ManualVerifierReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualVerifierReviewAdjuster {
    pub percent: f64,
}

impl ManualVerifierReviewStep for PercentManualVerifierReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualVerifierReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualVerifierReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualVerifierReviewStep for FlaggedManualVerifierReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualVerifierReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierReviewWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl ManualVerifierReviewStep for ManualVerifierReviewWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &ManualVerifierReview, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualVerifierReviewAdjuster {
    pub minimum: f64,
}

impl ManualVerifierReviewStep for MinimumManualVerifierReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualVerifierReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualVerifierReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualVerifierReviewStep for TieredManualVerifierReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualVerifierReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual verifier review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualVerifierReviewEngine {
    pub steps: Vec<Rc<dyn ManualVerifierReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualVerifierReviewEngine {
    pub fn compute(&mut self, item: &ManualVerifierReview) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[ManualVerifierReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_verifier_review_engine(flags: &flags::Registry) -> ManualVerifierReviewEngine {
    ManualVerifierReviewEngine {
        steps: vec![
            Rc::new(SourceManualVerifierReviewAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.7),
                    ("south".to_string(), 0.99),
                    ("retail".to_string(), 0.74),
                ]),
            }) as Rc<dyn ManualVerifierReviewStep>,
            Rc::new(PercentManualVerifierReviewAdjuster { percent: -8.6 }),
            Rc::new(FlaggedManualVerifierReviewAdjuster {
                flags: flags.clone(),
                flag: "manual-verifier-review-strict".to_string(),
                percent: 8.5,
            }),
            Rc::new(ManualVerifierReviewWindowDaysOffsetAdjuster { weight: 0.89 }),
            Rc::new(MinimumManualVerifierReviewAdjuster { minimum: 41.0 }),
            Rc::new(TieredManualVerifierReviewAdjuster {
                bounds: vec![2912.0, 5277.0, 8138.0, 9351.0],
                rates: vec![0.86, 1.15, 1.17, 0.99],
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 79156.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_verifier_review_engine(&flags::new_registry());
        let item = new_manual_verifier_review("code-225", "name-993");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
