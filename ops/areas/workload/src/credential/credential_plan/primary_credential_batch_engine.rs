use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary credential batch calculation.
pub trait PrimaryCredentialBatchStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryCredentialBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryCredentialBatchAdjuster {
    pub step: f64,
}

impl PrimaryCredentialBatchStep for RoundingPrimaryCredentialBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryCredentialBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryCredentialBatchAdjuster {
    pub percent: f64,
}

impl PrimaryCredentialBatchStep for PercentPrimaryCredentialBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryCredentialBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryCredentialBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryCredentialBatchStep for FlaggedPrimaryCredentialBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryCredentialBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPrimaryCredentialBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryCredentialBatchStep for LabelPrimaryCredentialBatchAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PrimaryCredentialBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryCredentialBatchAdjuster {
    pub cap: f64,
}

impl PrimaryCredentialBatchStep for CapPrimaryCredentialBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryCredentialBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryCredentialBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryCredentialBatchStep for TieredPrimaryCredentialBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryCredentialBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the primary credential batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryCredentialBatchEngine {
    pub steps: Vec<Rc<dyn PrimaryCredentialBatchStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryCredentialBatchEngine {
    pub fn price(&mut self, item: &PrimaryCredentialBatch) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryCredentialBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_credential_batch_engine(flags: &flags::Registry) -> PrimaryCredentialBatchEngine {
    PrimaryCredentialBatchEngine {
        steps: vec![
            Rc::new(RoundingPrimaryCredentialBatchAdjuster { step: 1.0 }) as Rc<dyn PrimaryCredentialBatchStep>,
            Rc::new(PercentPrimaryCredentialBatchAdjuster { percent: -15.5 }),
            Rc::new(FlaggedPrimaryCredentialBatchAdjuster {
                flags: flags.clone(),
                flag: "primary-credential-batch-regional".to_string(),
                percent: 4.2,
            }),
            Rc::new(LabelPrimaryCredentialBatchAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.26), ("retail".to_string(), 1.38)]),
            }),
            Rc::new(CapPrimaryCredentialBatchAdjuster { cap: 37714.0 }),
            Rc::new(TieredPrimaryCredentialBatchAdjuster {
                bounds: vec![133.0, 3518.0],
                rates: vec![1.1, 0.89],
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 47162.9,
    }
}
