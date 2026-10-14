use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched migration record calculation.
pub trait BatchedMigrationRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedMigrationRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedMigrationRecordAdjuster {
    pub cap: f64,
}

impl BatchedMigrationRecordStep for CapBatchedMigrationRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedMigrationRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedMigrationRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedMigrationRecordStep for TieredBatchedMigrationRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedMigrationRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedMigrationRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedMigrationRecordStep for FlaggedBatchedMigrationRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedMigrationRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedMigrationRecordAdjuster {
    pub percent: f64,
}

impl BatchedMigrationRecordStep for PercentBatchedMigrationRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedMigrationRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched migration record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedMigrationRecordEngine {
    pub steps: Vec<Rc<dyn BatchedMigrationRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedMigrationRecordEngine {
    pub fn evaluate(&mut self, item: &BatchedMigrationRecord) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BatchedMigrationRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_migration_record_engine(flags: &flags::Registry) -> BatchedMigrationRecordEngine {
    BatchedMigrationRecordEngine {
        steps: vec![
            Rc::new(CapBatchedMigrationRecordAdjuster { cap: 46937.0 }) as Rc<dyn BatchedMigrationRecordStep>,
            Rc::new(TieredBatchedMigrationRecordAdjuster {
                bounds: vec![823.0, 3320.0],
                rates: vec![1.06, 0.87],
            }),
            Rc::new(FlaggedBatchedMigrationRecordAdjuster {
                flags: flags.clone(),
                flag: "batched-migration-record-legacy".to_string(),
                percent: -5.9,
            }),
            Rc::new(PercentBatchedMigrationRecordAdjuster { percent: 1.6 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 45694.2,
    }
}
