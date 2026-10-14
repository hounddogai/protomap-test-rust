use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the failover record calculation.
pub trait FailoverRecordFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFailoverRecordAdjuster {
    pub percent: f64,
}

impl FailoverRecordFactor for PercentFailoverRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FailoverRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFailoverRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FailoverRecordFactor for FlaggedFailoverRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FailoverRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFailoverRecordAdjuster {
    pub minimum: f64,
}

impl FailoverRecordFactor for MinimumFailoverRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FailoverRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRecordEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl FailoverRecordFactor for FailoverRecordEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &FailoverRecord, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFailoverRecordAdjuster {
    pub cap: f64,
}

impl FailoverRecordFactor for CapFailoverRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FailoverRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitFailoverRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverRecordFactor for UnitFailoverRecordAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &FailoverRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the failover record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverRecordEngine {
    pub steps: Vec<Rc<dyn FailoverRecordFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverRecordEngine {
    pub fn price(&mut self, item: &FailoverRecord) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[FailoverRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_failover_record_engine(flags: &flags::Registry) -> FailoverRecordEngine {
    FailoverRecordEngine {
        steps: vec![
            Rc::new(PercentFailoverRecordAdjuster { percent: -2.5 }) as Rc<dyn FailoverRecordFactor>,
            Rc::new(FlaggedFailoverRecordAdjuster {
                flags: flags.clone(),
                flag: "failover-record-batched".to_string(),
                percent: 2.0,
            }),
            Rc::new(MinimumFailoverRecordAdjuster { minimum: 0.7 }),
            Rc::new(FailoverRecordEndsAtOffsetAdjuster { weight: 0.15 }),
            Rc::new(CapFailoverRecordAdjuster { cap: 36001.0 }),
            Rc::new(UnitFailoverRecordAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.19)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 60167.1,
    }
}
