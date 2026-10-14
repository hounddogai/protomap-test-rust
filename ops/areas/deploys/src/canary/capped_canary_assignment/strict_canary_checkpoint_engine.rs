use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the strict canary checkpoint calculation.
pub trait StrictCanaryCheckpointAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictCanaryCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryCheckpointCeilingOffsetAdjuster {
    pub weight: f64,
}

impl StrictCanaryCheckpointAdjuster for StrictCanaryCheckpointCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &StrictCanaryCheckpoint, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictCanaryCheckpointAdjuster {
    pub cap: f64,
}

impl StrictCanaryCheckpointAdjuster for CapStrictCanaryCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictCanaryCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictCanaryCheckpointAdjuster {
    pub minimum: f64,
}

impl StrictCanaryCheckpointAdjuster for MinimumStrictCanaryCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictCanaryCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictCanaryCheckpointAdjuster {
    pub percent: f64,
}

impl StrictCanaryCheckpointAdjuster for PercentStrictCanaryCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictCanaryCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictCanaryCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictCanaryCheckpointAdjuster for TieredStrictCanaryCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictCanaryCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the strict canary checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictCanaryCheckpointEngine {
    pub steps: Vec<Rc<dyn StrictCanaryCheckpointAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictCanaryCheckpointEngine {
    pub fn price(&mut self, item: &StrictCanaryCheckpoint) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictCanaryCheckpointAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictCanaryCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_canary_checkpoint_engine() -> StrictCanaryCheckpointEngine {
    StrictCanaryCheckpointEngine {
        steps: vec![
            Rc::new(StrictCanaryCheckpointCeilingOffsetAdjuster { weight: 1.17 })
                as Rc<dyn StrictCanaryCheckpointAdjuster>,
            Rc::new(CapStrictCanaryCheckpointAdjuster { cap: 42433.0 }),
            Rc::new(MinimumStrictCanaryCheckpointAdjuster { minimum: 43.1 }),
            Rc::new(PercentStrictCanaryCheckpointAdjuster { percent: -10.0 }),
            Rc::new(TieredStrictCanaryCheckpointAdjuster {
                bounds: vec![7928.0, 9330.0, 9434.0],
                rates: vec![1.08, 1.12, 1.12],
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 26524.5,
    }
}
