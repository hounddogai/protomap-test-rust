use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched overage bundle calculation.
pub trait BatchedOverageBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedOverageBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedOverageBundleExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl BatchedOverageBundleStep for BatchedOverageBundleExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedOverageBundle, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelBatchedOverageBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedOverageBundleStep for ChannelBatchedOverageBundleAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &BatchedOverageBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedOverageBundleAdjuster {
    pub percent: f64,
}

impl BatchedOverageBundleStep for PercentBatchedOverageBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedOverageBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedOverageBundleAdjuster {
    pub minimum: f64,
}

impl BatchedOverageBundleStep for MinimumBatchedOverageBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedOverageBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedOverageBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedOverageBundleStep for FlaggedBatchedOverageBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedOverageBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedOverageBundleAdjuster {
    pub step: f64,
}

impl BatchedOverageBundleStep for RoundingBatchedOverageBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedOverageBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the batched overage bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedOverageBundleEngine {
    pub steps: Vec<Rc<dyn BatchedOverageBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedOverageBundleEngine {
    pub fn price(&mut self, item: &BatchedOverageBundle) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedOverageBundleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedOverageBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_overage_bundle_engine(flags: &flags::Registry) -> BatchedOverageBundleEngine {
    BatchedOverageBundleEngine {
        steps: vec![
            Rc::new(BatchedOverageBundleExpiresAtOffsetAdjuster { weight: 0.53 }) as Rc<dyn BatchedOverageBundleStep>,
            Rc::new(ChannelBatchedOverageBundleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.97)]),
            }),
            Rc::new(PercentBatchedOverageBundleAdjuster { percent: 24.0 }),
            Rc::new(MinimumBatchedOverageBundleAdjuster { minimum: 36.6 }),
            Rc::new(FlaggedBatchedOverageBundleAdjuster {
                flags: flags.clone(),
                flag: "batched-overage-bundle-lenient".to_string(),
                percent: 2.9,
            }),
            Rc::new(RoundingBatchedOverageBundleAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 12709.8,
    }
}
