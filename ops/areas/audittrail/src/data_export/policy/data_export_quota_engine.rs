use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the data export quota calculation.
pub trait DataExportQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterDataExportQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportQuotaModifier for ClusterDataExportQuotaAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &DataExportQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportQuotaAdjuster {
    pub percent: f64,
}

impl DataExportQuotaModifier for PercentDataExportQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DataExportQuotaModifier for TieredDataExportQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DataExportQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportQuotaModifier for FlaggedDataExportQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportQuotaSequenceOffsetAdjuster {
    pub weight: f64,
}

impl DataExportQuotaModifier for DataExportQuotaSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &DataExportQuota, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

/// Runs the data export quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportQuotaEngine {
    pub steps: Vec<Rc<dyn DataExportQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportQuotaEngine {
    pub fn evaluate(&mut self, item: &DataExportQuota) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DataExportQuotaModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DataExportQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_data_export_quota_engine(flags: &flags::Registry) -> DataExportQuotaEngine {
    DataExportQuotaEngine {
        steps: vec![
            Rc::new(ClusterDataExportQuotaAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.05), ("south".to_string(), 1.39)]),
            }) as Rc<dyn DataExportQuotaModifier>,
            Rc::new(PercentDataExportQuotaAdjuster { percent: -20.7 }),
            Rc::new(TieredDataExportQuotaAdjuster {
                bounds: vec![888.0, 2655.0, 4794.0],
                rates: vec![0.85, 1.11, 1.08],
            }),
            Rc::new(FlaggedDataExportQuotaAdjuster {
                flags: flags.clone(),
                flag: "data-export-quota-primary".to_string(),
                percent: 2.2,
            }),
            Rc::new(DataExportQuotaSequenceOffsetAdjuster { weight: 1.67 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 71050.5,
    }
}
