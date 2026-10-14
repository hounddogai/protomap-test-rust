use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient nonce record calculation.
pub trait LenientNonceRecordAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientNonceRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientNonceRecordAdjuster {
    pub minimum: f64,
}

impl LenientNonceRecordAdjuster for MinimumLenientNonceRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientNonceRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceRecordAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl LenientNonceRecordAdjuster for LenientNonceRecordAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &LenientNonceRecord, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientNonceRecordAdjuster {
    pub cap: f64,
}

impl LenientNonceRecordAdjuster for CapLenientNonceRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientNonceRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientNonceRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientNonceRecordAdjuster for FlaggedLenientNonceRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientNonceRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientNonceRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientNonceRecordAdjuster for TieredLenientNonceRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientNonceRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the lenient nonce record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientNonceRecordEngine {
    pub steps: Vec<Rc<dyn LenientNonceRecordAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientNonceRecordEngine {
    pub fn price(&mut self, item: &LenientNonceRecord) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientNonceRecordAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientNonceRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_nonce_record_engine(flags: &flags::Registry) -> LenientNonceRecordEngine {
    LenientNonceRecordEngine {
        steps: vec![
            Rc::new(MinimumLenientNonceRecordAdjuster { minimum: 26.1 }) as Rc<dyn LenientNonceRecordAdjuster>,
            Rc::new(LenientNonceRecordAttemptsOffsetAdjuster { weight: 1.21 }),
            Rc::new(CapLenientNonceRecordAdjuster { cap: 36669.0 }),
            Rc::new(FlaggedLenientNonceRecordAdjuster {
                flags: flags.clone(),
                flag: "lenient-nonce-record-primary".to_string(),
                percent: -3.9,
            }),
            Rc::new(TieredLenientNonceRecordAdjuster {
                bounds: vec![158.0, 5825.0, 6941.0, 9927.0],
                rates: vec![1.02, 1.0, 1.16, 0.91],
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 40734.6,
    }
}
