use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary flow log record calculation.
pub trait PrimaryFlowLogRecordFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryFlowLogRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryFlowLogRecordAdjuster {
    pub minimum: f64,
}

impl PrimaryFlowLogRecordFactor for MinimumPrimaryFlowLogRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryFlowLogRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryFlowLogRecordAdjuster {
    pub cap: f64,
}

impl PrimaryFlowLogRecordFactor for CapPrimaryFlowLogRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryFlowLogRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryFlowLogRecordAdjuster {
    pub step: f64,
}

impl PrimaryFlowLogRecordFactor for RoundingPrimaryFlowLogRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryFlowLogRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryFlowLogRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryFlowLogRecordFactor for FlaggedPrimaryFlowLogRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryFlowLogRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary flow log record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryFlowLogRecordEngine {
    pub steps: Vec<Rc<dyn PrimaryFlowLogRecordFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryFlowLogRecordEngine {
    pub fn compute(&mut self, item: &PrimaryFlowLogRecord) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryFlowLogRecordFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryFlowLogRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_flow_log_record_engine(flags: &flags::Registry) -> PrimaryFlowLogRecordEngine {
    PrimaryFlowLogRecordEngine {
        steps: vec![
            Rc::new(MinimumPrimaryFlowLogRecordAdjuster { minimum: 13.8 }) as Rc<dyn PrimaryFlowLogRecordFactor>,
            Rc::new(CapPrimaryFlowLogRecordAdjuster { cap: 23179.0 }),
            Rc::new(RoundingPrimaryFlowLogRecordAdjuster { step: 0.25 }),
            Rc::new(FlaggedPrimaryFlowLogRecordAdjuster {
                flags: flags.clone(),
                flag: "primary-flow-log-record-manual".to_string(),
                percent: 7.5,
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 86452.4,
    }
}
