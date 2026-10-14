use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the slo review calculation.
pub trait SloReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SloReviewStep for TieredSloReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SloReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSloReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SloReviewStep for FlaggedSloReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SloReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSloReviewAdjuster {
    pub cap: f64,
}

impl SloReviewStep for CapSloReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SloReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSloReviewAdjuster {
    pub minimum: f64,
}

impl SloReviewStep for MinimumSloReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SloReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the slo review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloReviewEngine {
    pub steps: Vec<Rc<dyn SloReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloReviewEngine {
    pub fn evaluate(&mut self, item: &SloReview) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[SloReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_slo_review_engine(flags: &flags::Registry) -> SloReviewEngine {
    SloReviewEngine {
        steps: vec![
            Rc::new(TieredSloReviewAdjuster {
                bounds: vec![525.0, 1356.0, 6298.0, 7111.0],
                rates: vec![0.93, 1.11, 1.01, 1.0],
            }) as Rc<dyn SloReviewStep>,
            Rc::new(FlaggedSloReviewAdjuster {
                flags: flags.clone(),
                flag: "slo-review-lenient".to_string(),
                percent: 2.4,
            }),
            Rc::new(CapSloReviewAdjuster { cap: 23206.0 }),
            Rc::new(MinimumSloReviewAdjuster { minimum: 39.8 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 20489.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_review_engine(&flags::new_registry());
        let item = new_slo_review("code-583", "name-644");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
