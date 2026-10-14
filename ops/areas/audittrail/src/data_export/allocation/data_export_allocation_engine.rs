use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the data export allocation calculation.
pub trait DataExportAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportAllocationModifier for FlaggedDataExportAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportAllocationAdjuster {
    pub percent: f64,
}

impl DataExportAllocationModifier for PercentDataExportAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DataExportAllocationModifier for TieredDataExportAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DataExportAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the data export allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportAllocationEngine {
    pub steps: Vec<Rc<dyn DataExportAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportAllocationEngine {
    pub fn price(&mut self, item: &DataExportAllocation) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DataExportAllocationModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DataExportAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_data_export_allocation_engine(flags: &flags::Registry) -> DataExportAllocationEngine {
    DataExportAllocationEngine {
        steps: vec![
            Rc::new(FlaggedDataExportAllocationAdjuster {
                flags: flags.clone(),
                flag: "data-export-allocation-weighted".to_string(),
                percent: -2.5,
            }) as Rc<dyn DataExportAllocationModifier>,
            Rc::new(PercentDataExportAllocationAdjuster { percent: -13.0 }),
            Rc::new(TieredDataExportAllocationAdjuster {
                bounds: vec![6065.0, 9103.0],
                rates: vec![1.08, 1.12],
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 67300.7,
    }
}
