use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched postmortem profile calculation.
pub trait BatchedPostmortemProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedPostmortemProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelBatchedPostmortemProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedPostmortemProfileStep for ChannelBatchedPostmortemProfileAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &BatchedPostmortemProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedPostmortemProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedPostmortemProfileStep for TieredBatchedPostmortemProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedPostmortemProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedPostmortemProfileAdjuster {
    pub step: f64,
}

impl BatchedPostmortemProfileStep for RoundingBatchedPostmortemProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedPostmortemProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedPostmortemProfileAdjuster {
    pub minimum: f64,
}

impl BatchedPostmortemProfileStep for MinimumBatchedPostmortemProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedPostmortemProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedPostmortemProfileAdjuster {
    pub percent: f64,
}

impl BatchedPostmortemProfileStep for PercentBatchedPostmortemProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedPostmortemProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedPostmortemProfileAdjuster {
    pub cap: f64,
}

impl BatchedPostmortemProfileStep for CapBatchedPostmortemProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedPostmortemProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the batched postmortem profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedPostmortemProfileEngine {
    pub steps: Vec<Rc<dyn BatchedPostmortemProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedPostmortemProfileEngine {
    pub fn run(&mut self, item: &BatchedPostmortemProfile) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[BatchedPostmortemProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_postmortem_profile_engine() -> BatchedPostmortemProfileEngine {
    BatchedPostmortemProfileEngine {
        steps: vec![
            Rc::new(ChannelBatchedPostmortemProfileAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.92)]),
            }) as Rc<dyn BatchedPostmortemProfileStep>,
            Rc::new(TieredBatchedPostmortemProfileAdjuster {
                bounds: vec![1035.0, 4081.0, 5799.0, 6001.0],
                rates: vec![1.15, 0.89, 1.17, 1.0],
            }),
            Rc::new(RoundingBatchedPostmortemProfileAdjuster { step: 0.25 }),
            Rc::new(MinimumBatchedPostmortemProfileAdjuster { minimum: 25.6 }),
            Rc::new(PercentBatchedPostmortemProfileAdjuster { percent: 9.5 }),
            Rc::new(CapBatchedPostmortemProfileAdjuster { cap: 36193.0 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 81940.8,
    }
}
