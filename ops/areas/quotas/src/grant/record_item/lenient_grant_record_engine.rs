use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient grant record calculation.
pub trait LenientGrantRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientGrantRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientGrantRecordAdjuster {
    pub minimum: f64,
}

impl LenientGrantRecordStep for MinimumLenientGrantRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientGrantRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientGrantRecordAdjuster {
    pub cap: f64,
}

impl LenientGrantRecordStep for CapLenientGrantRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientGrantRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientGrantRecordAdjuster {
    pub step: f64,
}

impl LenientGrantRecordStep for RoundingLenientGrantRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientGrantRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantRecordQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl LenientGrantRecordStep for LenientGrantRecordQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &LenientGrantRecord, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientGrantRecordAdjuster {
    pub percent: f64,
}

impl LenientGrantRecordStep for PercentLenientGrantRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientGrantRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientGrantRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientGrantRecordStep for FlaggedLenientGrantRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientGrantRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient grant record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientGrantRecordEngine {
    pub steps: Vec<Rc<dyn LenientGrantRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientGrantRecordEngine {
    pub fn compute(&mut self, item: &LenientGrantRecord) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientGrantRecordStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientGrantRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_grant_record_engine(flags: &flags::Registry) -> LenientGrantRecordEngine {
    LenientGrantRecordEngine {
        steps: vec![
            Rc::new(MinimumLenientGrantRecordAdjuster { minimum: 20.1 }) as Rc<dyn LenientGrantRecordStep>,
            Rc::new(CapLenientGrantRecordAdjuster { cap: 46349.0 }),
            Rc::new(RoundingLenientGrantRecordAdjuster { step: 0.25 }),
            Rc::new(LenientGrantRecordQuotaUnitsOffsetAdjuster { weight: 0.17 }),
            Rc::new(PercentLenientGrantRecordAdjuster { percent: -9.0 }),
            Rc::new(FlaggedLenientGrantRecordAdjuster {
                flags: flags.clone(),
                flag: "lenient-grant-record-rolling".to_string(),
                percent: -7.0,
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 38761.1,
    }
}
