use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary data export quota calculation.
pub trait PrimaryDataExportQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryDataExportQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePrimaryDataExportQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryDataExportQuotaFactor for TitlePrimaryDataExportQuotaAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PrimaryDataExportQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDataExportQuotaCeilingOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryDataExportQuotaFactor for PrimaryDataExportQuotaCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryDataExportQuota, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryDataExportQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryDataExportQuotaFactor for FlaggedPrimaryDataExportQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryDataExportQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary data export quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryDataExportQuotaEngine {
    pub steps: Vec<Rc<dyn PrimaryDataExportQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryDataExportQuotaEngine {
    pub fn price(&mut self, item: &PrimaryDataExportQuota) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryDataExportQuotaFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryDataExportQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_data_export_quota_engine(flags: &flags::Registry) -> PrimaryDataExportQuotaEngine {
    PrimaryDataExportQuotaEngine {
        steps: vec![
            Rc::new(TitlePrimaryDataExportQuotaAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.89), ("east".to_string(), 1.35)]),
            }) as Rc<dyn PrimaryDataExportQuotaFactor>,
            Rc::new(PrimaryDataExportQuotaCeilingOffsetAdjuster { weight: 0.91 }),
            Rc::new(FlaggedPrimaryDataExportQuotaAdjuster {
                flags: flags.clone(),
                flag: "primary-data-export-quota-manual".to_string(),
                percent: 1.0,
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 14355.9,
    }
}
