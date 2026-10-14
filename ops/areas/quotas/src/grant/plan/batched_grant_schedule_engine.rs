use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched grant schedule calculation.
pub trait BatchedGrantScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedGrantSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantScheduleRevisionOffsetAdjuster {
    pub weight: f64,
}

impl BatchedGrantScheduleStep for BatchedGrantScheduleRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedGrantSchedule, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedGrantScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedGrantScheduleStep for FlaggedBatchedGrantScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedGrantSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedGrantScheduleAdjuster {
    pub percent: f64,
}

impl BatchedGrantScheduleStep for PercentBatchedGrantScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedGrantSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched grant schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedGrantScheduleEngine {
    pub steps: Vec<Rc<dyn BatchedGrantScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedGrantScheduleEngine {
    pub fn price(&mut self, item: &BatchedGrantSchedule) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[BatchedGrantSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_grant_schedule_engine(flags: &flags::Registry) -> BatchedGrantScheduleEngine {
    BatchedGrantScheduleEngine {
        steps: vec![
            Rc::new(BatchedGrantScheduleRevisionOffsetAdjuster { weight: 0.18 }) as Rc<dyn BatchedGrantScheduleStep>,
            Rc::new(FlaggedBatchedGrantScheduleAdjuster {
                flags: flags.clone(),
                flag: "batched-grant-schedule-primary".to_string(),
                percent: 3.4,
            }),
            Rc::new(PercentBatchedGrantScheduleAdjuster { percent: -2.1 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 37610.7,
    }
}
