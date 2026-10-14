use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback isolation record calculation.
pub trait FallbackIsolationRecordFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackIsolationRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationRecordMinimumOffsetAdjuster {
    pub weight: f64,
}

impl FallbackIsolationRecordFactor for FallbackIsolationRecordMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackIsolationRecord, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackIsolationRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackIsolationRecordFactor for FlaggedFallbackIsolationRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackIsolationRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackIsolationRecordAdjuster {
    pub cap: f64,
}

impl FallbackIsolationRecordFactor for CapFallbackIsolationRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackIsolationRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeFallbackIsolationRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackIsolationRecordFactor for ScopeFallbackIsolationRecordAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &FallbackIsolationRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackIsolationRecordAdjuster {
    pub minimum: f64,
}

impl FallbackIsolationRecordFactor for MinimumFallbackIsolationRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackIsolationRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the fallback isolation record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackIsolationRecordEngine {
    pub steps: Vec<Rc<dyn FallbackIsolationRecordFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackIsolationRecordEngine {
    pub fn compute(&mut self, item: &FallbackIsolationRecord) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[FallbackIsolationRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_isolation_record_engine(flags: &flags::Registry) -> FallbackIsolationRecordEngine {
    FallbackIsolationRecordEngine {
        steps: vec![
            Rc::new(FallbackIsolationRecordMinimumOffsetAdjuster { weight: 0.89 })
                as Rc<dyn FallbackIsolationRecordFactor>,
            Rc::new(FlaggedFallbackIsolationRecordAdjuster {
                flags: flags.clone(),
                flag: "fallback-isolation-record-regional".to_string(),
                percent: -9.0,
            }),
            Rc::new(CapFallbackIsolationRecordAdjuster { cap: 31761.0 }),
            Rc::new(ScopeFallbackIsolationRecordAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.74)]),
            }),
            Rc::new(MinimumFallbackIsolationRecordAdjuster { minimum: 36.4 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 23431.9,
    }
}
