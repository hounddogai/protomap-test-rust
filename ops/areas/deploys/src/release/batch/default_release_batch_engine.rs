use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default release batch calculation.
pub trait DefaultReleaseBatchAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultReleaseBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultReleaseBatchAdjuster {
    pub cap: f64,
}

impl DefaultReleaseBatchAdjuster for CapDefaultReleaseBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultReleaseBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultReleaseBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultReleaseBatchAdjuster for FlaggedDefaultReleaseBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultReleaseBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultReleaseBatchAdjuster {
    pub minimum: f64,
}

impl DefaultReleaseBatchAdjuster for MinimumDefaultReleaseBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultReleaseBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultReleaseBatchAdjuster {
    pub step: f64,
}

impl DefaultReleaseBatchAdjuster for RoundingDefaultReleaseBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultReleaseBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultReleaseBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultReleaseBatchAdjuster for TieredDefaultReleaseBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultReleaseBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the default release batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultReleaseBatchEngine {
    pub steps: Vec<Rc<dyn DefaultReleaseBatchAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultReleaseBatchEngine {
    pub fn run(&mut self, item: &DefaultReleaseBatch) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultReleaseBatchAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultReleaseBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_release_batch_engine(flags: &flags::Registry) -> DefaultReleaseBatchEngine {
    DefaultReleaseBatchEngine {
        steps: vec![
            Rc::new(CapDefaultReleaseBatchAdjuster { cap: 2342.0 }) as Rc<dyn DefaultReleaseBatchAdjuster>,
            Rc::new(FlaggedDefaultReleaseBatchAdjuster {
                flags: flags.clone(),
                flag: "default-release-batch-weighted".to_string(),
                percent: 3.4,
            }),
            Rc::new(MinimumDefaultReleaseBatchAdjuster { minimum: 31.6 }),
            Rc::new(RoundingDefaultReleaseBatchAdjuster { step: 0.1 }),
            Rc::new(TieredDefaultReleaseBatchAdjuster {
                bounds: vec![262.0, 1583.0],
                rates: vec![1.14, 1.11],
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 65247.6,
    }
}
