use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending alert batch calculation.
pub trait PendingAlertBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingAlertBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelPendingAlertBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingAlertBatchFactor for ChannelPendingAlertBatchAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &PendingAlertBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingAlertBatchAdjuster {
    pub step: f64,
}

impl PendingAlertBatchFactor for RoundingPendingAlertBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingAlertBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingAlertBatchAdjuster {
    pub cap: f64,
}

impl PendingAlertBatchFactor for CapPendingAlertBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingAlertBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the pending alert batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingAlertBatchEngine {
    pub steps: Vec<Rc<dyn PendingAlertBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingAlertBatchEngine {
    pub fn compute(&mut self, item: &PendingAlertBatch) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingAlertBatchFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingAlertBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_alert_batch_engine() -> PendingAlertBatchEngine {
    PendingAlertBatchEngine {
        steps: vec![
            Rc::new(ChannelPendingAlertBatchAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.87)]),
            }) as Rc<dyn PendingAlertBatchFactor>,
            Rc::new(RoundingPendingAlertBatchAdjuster { step: 5.0 }),
            Rc::new(CapPendingAlertBatchAdjuster { cap: 45833.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 49972.7,
    }
}
