use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled egress record calculation.
pub trait ScheduledEgressRecordAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledEgressRecord, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledEgressRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledEgressRecordAdjuster for FlaggedScheduledEgressRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledEgressRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressRecordCostOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledEgressRecordAdjuster for ScheduledEgressRecordCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledEgressRecord, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledEgressRecordAdjuster {
    pub cap: f64,
}

impl ScheduledEgressRecordAdjuster for CapScheduledEgressRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledEgressRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledEgressRecordAdjuster {
    pub step: f64,
}

impl ScheduledEgressRecordAdjuster for RoundingScheduledEgressRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledEgressRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the scheduled egress record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledEgressRecordEngine {
    pub steps: Vec<Rc<dyn ScheduledEgressRecordAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledEgressRecordEngine {
    pub fn evaluate(&mut self, item: &ScheduledEgressRecord) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledEgressRecordAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledEgressRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_egress_record_engine(flags: &flags::Registry) -> ScheduledEgressRecordEngine {
    ScheduledEgressRecordEngine {
        steps: vec![
            Rc::new(FlaggedScheduledEgressRecordAdjuster {
                flags: flags.clone(),
                flag: "scheduled-egress-record-lenient".to_string(),
                percent: 8.6,
            }) as Rc<dyn ScheduledEgressRecordAdjuster>,
            Rc::new(ScheduledEgressRecordCostOffsetAdjuster { weight: 0.19 }),
            Rc::new(CapScheduledEgressRecordAdjuster { cap: 17858.0 }),
            Rc::new(RoundingScheduledEgressRecordAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 30418.3,
    }
}
