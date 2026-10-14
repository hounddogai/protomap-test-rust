use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the sink review calculation.
pub trait SinkReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSinkReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkReviewModifier for LabelSinkReviewAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &SinkReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSinkReviewAdjuster {
    pub cap: f64,
}

impl SinkReviewModifier for CapSinkReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SinkReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSinkReviewAdjuster {
    pub percent: f64,
}

impl SinkReviewModifier for PercentSinkReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SinkReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSinkReviewAdjuster {
    pub step: f64,
}

impl SinkReviewModifier for RoundingSinkReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SinkReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SinkReviewModifier for TieredSinkReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SinkReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSinkReviewAdjuster {
    pub minimum: f64,
}

impl SinkReviewModifier for MinimumSinkReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SinkReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the sink review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkReviewEngine {
    pub steps: Vec<Rc<dyn SinkReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkReviewEngine {
    pub fn price(&mut self, item: &SinkReview) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SinkReviewModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SinkReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_sink_review_engine() -> SinkReviewEngine {
    SinkReviewEngine {
        steps: vec![
            Rc::new(LabelSinkReviewAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.29),
                    ("east".to_string(), 0.84),
                    ("online".to_string(), 1.16),
                ]),
            }) as Rc<dyn SinkReviewModifier>,
            Rc::new(CapSinkReviewAdjuster { cap: 4933.0 }),
            Rc::new(PercentSinkReviewAdjuster { percent: -17.7 }),
            Rc::new(RoundingSinkReviewAdjuster { step: 5.0 }),
            Rc::new(TieredSinkReviewAdjuster {
                bounds: vec![3059.0, 7521.0, 9296.0],
                rates: vec![1.02, 1.16, 1.13],
            }),
            Rc::new(MinimumSinkReviewAdjuster { minimum: 2.5 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 18398.0,
    }
}
