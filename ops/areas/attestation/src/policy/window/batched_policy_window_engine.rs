use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched policy window calculation.
pub trait BatchedPolicyWindowAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedPolicyWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedPolicyWindowAdjuster {
    pub minimum: f64,
}

impl BatchedPolicyWindowAdjuster for MinimumBatchedPolicyWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedPolicyWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedPolicyWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedPolicyWindowAdjuster for FlaggedBatchedPolicyWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedPolicyWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyWindowBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl BatchedPolicyWindowAdjuster for BatchedPolicyWindowBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedPolicyWindow, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedPolicyWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedPolicyWindowAdjuster for TieredBatchedPolicyWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedPolicyWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedPolicyWindowAdjuster {
    pub cap: f64,
}

impl BatchedPolicyWindowAdjuster for CapBatchedPolicyWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedPolicyWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleBatchedPolicyWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedPolicyWindowAdjuster for LocaleBatchedPolicyWindowAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &BatchedPolicyWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the batched policy window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedPolicyWindowEngine {
    pub steps: Vec<Rc<dyn BatchedPolicyWindowAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedPolicyWindowEngine {
    pub fn compute(&mut self, item: &BatchedPolicyWindow) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedPolicyWindowAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedPolicyWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_policy_window_engine(flags: &flags::Registry) -> BatchedPolicyWindowEngine {
    BatchedPolicyWindowEngine {
        steps: vec![
            Rc::new(MinimumBatchedPolicyWindowAdjuster { minimum: 34.1 }) as Rc<dyn BatchedPolicyWindowAdjuster>,
            Rc::new(FlaggedBatchedPolicyWindowAdjuster {
                flags: flags.clone(),
                flag: "batched-policy-window-regional".to_string(),
                percent: 5.5,
            }),
            Rc::new(BatchedPolicyWindowBatchSizeOffsetAdjuster { weight: 1.03 }),
            Rc::new(TieredBatchedPolicyWindowAdjuster {
                bounds: vec![3531.0, 4053.0, 5719.0, 7092.0],
                rates: vec![1.17, 1.0, 0.88, 1.1],
            }),
            Rc::new(CapBatchedPolicyWindowAdjuster { cap: 9851.0 }),
            Rc::new(LocaleBatchedPolicyWindowAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.17),
                    ("retail".to_string(), 1.3),
                    ("central".to_string(), 0.89),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 75786.1,
    }
}
