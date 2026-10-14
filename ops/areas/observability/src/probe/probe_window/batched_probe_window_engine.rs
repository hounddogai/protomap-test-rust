use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched probe window calculation.
pub trait BatchedProbeWindowFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedProbeWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedProbeWindowAdjuster {
    pub step: f64,
}

impl BatchedProbeWindowFactor for RoundingBatchedProbeWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedProbeWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedProbeWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedProbeWindowFactor for FlaggedBatchedProbeWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedProbeWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedProbeWindowAdjuster {
    pub minimum: f64,
}

impl BatchedProbeWindowFactor for MinimumBatchedProbeWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedProbeWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeWindowPortOffsetAdjuster {
    pub weight: f64,
}

impl BatchedProbeWindowFactor for BatchedProbeWindowPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedProbeWindow, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedProbeWindowAdjuster {
    pub cap: f64,
}

impl BatchedProbeWindowFactor for CapBatchedProbeWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedProbeWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBatchedProbeWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedProbeWindowFactor for NameBatchedProbeWindowAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BatchedProbeWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the batched probe window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedProbeWindowEngine {
    pub steps: Vec<Rc<dyn BatchedProbeWindowFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedProbeWindowEngine {
    pub fn evaluate(&mut self, item: &BatchedProbeWindow) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedProbeWindowFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedProbeWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_probe_window_engine(flags: &flags::Registry) -> BatchedProbeWindowEngine {
    BatchedProbeWindowEngine {
        steps: vec![
            Rc::new(RoundingBatchedProbeWindowAdjuster { step: 5.0 }) as Rc<dyn BatchedProbeWindowFactor>,
            Rc::new(FlaggedBatchedProbeWindowAdjuster {
                flags: flags.clone(),
                flag: "batched-probe-window-default".to_string(),
                percent: 6.0,
            }),
            Rc::new(MinimumBatchedProbeWindowAdjuster { minimum: 49.6 }),
            Rc::new(BatchedProbeWindowPortOffsetAdjuster { weight: 1.39 }),
            Rc::new(CapBatchedProbeWindowAdjuster { cap: 41422.0 }),
            Rc::new(NameBatchedProbeWindowAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.08),
                    ("south".to_string(), 1.24),
                    ("retail".to_string(), 0.96),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 70143.2,
    }
}
