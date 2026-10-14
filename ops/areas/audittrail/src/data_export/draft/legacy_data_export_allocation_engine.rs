use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy data export allocation calculation.
pub trait LegacyDataExportAllocationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyDataExportAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyDataExportAllocationAdjuster {
    pub percent: f64,
}

impl LegacyDataExportAllocationAdjuster for PercentLegacyDataExportAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyDataExportAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyDataExportAllocationAdjuster {
    pub step: f64,
}

impl LegacyDataExportAllocationAdjuster for RoundingLegacyDataExportAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyDataExportAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyDataExportAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyDataExportAllocationAdjuster for FlaggedLegacyDataExportAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyDataExportAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLegacyDataExportAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyDataExportAllocationAdjuster for SourceLegacyDataExportAllocationAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &LegacyDataExportAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyDataExportAllocationAdjuster {
    pub minimum: f64,
}

impl LegacyDataExportAllocationAdjuster for MinimumLegacyDataExportAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyDataExportAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportAllocationRetriesOffsetAdjuster {
    pub weight: f64,
}

impl LegacyDataExportAllocationAdjuster for LegacyDataExportAllocationRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyDataExportAllocation, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

/// Runs the legacy data export allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyDataExportAllocationEngine {
    pub steps: Vec<Rc<dyn LegacyDataExportAllocationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyDataExportAllocationEngine {
    pub fn price(&mut self, item: &LegacyDataExportAllocation) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyDataExportAllocationAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyDataExportAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_data_export_allocation_engine(flags: &flags::Registry) -> LegacyDataExportAllocationEngine {
    LegacyDataExportAllocationEngine {
        steps: vec![
            Rc::new(PercentLegacyDataExportAllocationAdjuster { percent: 4.0 })
                as Rc<dyn LegacyDataExportAllocationAdjuster>,
            Rc::new(RoundingLegacyDataExportAllocationAdjuster { step: 5.0 }),
            Rc::new(FlaggedLegacyDataExportAllocationAdjuster {
                flags: flags.clone(),
                flag: "legacy-data-export-allocation-seasonal".to_string(),
                percent: -4.5,
            }),
            Rc::new(SourceLegacyDataExportAllocationAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.84), ("north".to_string(), 0.99)]),
            }),
            Rc::new(MinimumLegacyDataExportAllocationAdjuster { minimum: 31.7 }),
            Rc::new(LegacyDataExportAllocationRetriesOffsetAdjuster { weight: 1.83 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 22427.2,
    }
}
