use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict segment window calculation.
pub trait StrictSegmentWindowModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictSegmentWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameStrictSegmentWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictSegmentWindowModifier for NameStrictSegmentWindowAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &StrictSegmentWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictSegmentWindowAdjuster {
    pub step: f64,
}

impl StrictSegmentWindowModifier for RoundingStrictSegmentWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictSegmentWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictSegmentWindowAdjuster {
    pub cap: f64,
}

impl StrictSegmentWindowModifier for CapStrictSegmentWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictSegmentWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentWindowAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl StrictSegmentWindowModifier for StrictSegmentWindowAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &StrictSegmentWindow, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictSegmentWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictSegmentWindowModifier for TieredStrictSegmentWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictSegmentWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictSegmentWindowAdjuster {
    pub minimum: f64,
}

impl StrictSegmentWindowModifier for MinimumStrictSegmentWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictSegmentWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the strict segment window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictSegmentWindowEngine {
    pub steps: Vec<Rc<dyn StrictSegmentWindowModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictSegmentWindowEngine {
    pub fn evaluate(&mut self, item: &StrictSegmentWindow) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictSegmentWindowModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictSegmentWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_segment_window_engine() -> StrictSegmentWindowEngine {
    StrictSegmentWindowEngine {
        steps: vec![
            Rc::new(NameStrictSegmentWindowAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.39)]),
            }) as Rc<dyn StrictSegmentWindowModifier>,
            Rc::new(RoundingStrictSegmentWindowAdjuster { step: 0.05 }),
            Rc::new(CapStrictSegmentWindowAdjuster { cap: 13471.0 }),
            Rc::new(StrictSegmentWindowAttemptsOffsetAdjuster { weight: 1.36 }),
            Rc::new(TieredStrictSegmentWindowAdjuster {
                bounds: vec![3762.0, 5903.0],
                rates: vec![0.9, 0.88],
            }),
            Rc::new(MinimumStrictSegmentWindowAdjuster { minimum: 40.1 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 36237.2,
    }
}
