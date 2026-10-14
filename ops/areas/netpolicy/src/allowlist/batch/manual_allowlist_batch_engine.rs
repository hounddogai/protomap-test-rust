use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual allowlist batch calculation.
pub trait ManualAllowlistBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualAllowlistBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistBatchTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl ManualAllowlistBatchFactor for ManualAllowlistBatchTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &ManualAllowlistBatch, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualAllowlistBatchAdjuster {
    pub cap: f64,
}

impl ManualAllowlistBatchFactor for CapManualAllowlistBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualAllowlistBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualAllowlistBatchAdjuster {
    pub percent: f64,
}

impl ManualAllowlistBatchFactor for PercentManualAllowlistBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualAllowlistBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualAllowlistBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualAllowlistBatchFactor for FlaggedManualAllowlistBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualAllowlistBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the manual allowlist batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualAllowlistBatchEngine {
    pub steps: Vec<Rc<dyn ManualAllowlistBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualAllowlistBatchEngine {
    pub fn price(&mut self, item: &ManualAllowlistBatch) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[ManualAllowlistBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_allowlist_batch_engine(flags: &flags::Registry) -> ManualAllowlistBatchEngine {
    ManualAllowlistBatchEngine {
        steps: vec![
            Rc::new(ManualAllowlistBatchTtlSecondsOffsetAdjuster { weight: 1.24 })
                as Rc<dyn ManualAllowlistBatchFactor>,
            Rc::new(CapManualAllowlistBatchAdjuster { cap: 9142.0 }),
            Rc::new(PercentManualAllowlistBatchAdjuster { percent: 20.4 }),
            Rc::new(FlaggedManualAllowlistBatchAdjuster {
                flags: flags.clone(),
                flag: "manual-allowlist-batch-tiered".to_string(),
                percent: -6.3,
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 89744.6,
    }
}
