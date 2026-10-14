use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict verifier segment calculation.
pub trait StrictVerifierSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictVerifierSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictVerifierSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictVerifierSegmentStep for TieredStrictVerifierSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictVerifierSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictVerifierSegmentAdjuster {
    pub minimum: f64,
}

impl StrictVerifierSegmentStep for MinimumStrictVerifierSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictVerifierSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictVerifierSegmentAdjuster {
    pub step: f64,
}

impl StrictVerifierSegmentStep for RoundingStrictVerifierSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictVerifierSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictVerifierSegmentScoreOffsetAdjuster {
    pub weight: f64,
}

impl StrictVerifierSegmentStep for StrictVerifierSegmentScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &StrictVerifierSegment, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictVerifierSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictVerifierSegmentStep for FlaggedStrictVerifierSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictVerifierSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict verifier segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictVerifierSegmentEngine {
    pub steps: Vec<Rc<dyn StrictVerifierSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictVerifierSegmentEngine {
    pub fn compute(&mut self, item: &StrictVerifierSegment) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictVerifierSegmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictVerifierSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_verifier_segment_engine(flags: &flags::Registry) -> StrictVerifierSegmentEngine {
    StrictVerifierSegmentEngine {
        steps: vec![
            Rc::new(TieredStrictVerifierSegmentAdjuster {
                bounds: vec![1531.0, 4289.0, 9662.0],
                rates: vec![1.02, 1.18, 1.04],
            }) as Rc<dyn StrictVerifierSegmentStep>,
            Rc::new(MinimumStrictVerifierSegmentAdjuster { minimum: 47.6 }),
            Rc::new(RoundingStrictVerifierSegmentAdjuster { step: 0.25 }),
            Rc::new(StrictVerifierSegmentScoreOffsetAdjuster { weight: 1.24 }),
            Rc::new(FlaggedStrictVerifierSegmentAdjuster {
                flags: flags.clone(),
                flag: "strict-verifier-segment-lenient".to_string(),
                percent: -2.9,
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 48419.7,
    }
}
