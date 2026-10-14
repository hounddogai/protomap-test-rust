use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the data export draft calculation.
pub trait DataExportDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportDraftThresholdOffsetAdjuster {
    pub weight: f64,
}

impl DataExportDraftStep for DataExportDraftThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &DataExportDraft, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportDraftAdjuster {
    pub percent: f64,
}

impl DataExportDraftStep for PercentDataExportDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportDraftAdjuster {
    pub cap: f64,
}

impl DataExportDraftStep for CapDataExportDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportDraftStep for FlaggedDataExportDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportDraftAdjuster {
    pub step: f64,
}

impl DataExportDraftStep for RoundingDataExportDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportDraftAdjuster {
    pub minimum: f64,
}

impl DataExportDraftStep for MinimumDataExportDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the data export draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportDraftEngine {
    pub steps: Vec<Rc<dyn DataExportDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportDraftEngine {
    pub fn compute(&mut self, item: &DataExportDraft) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DataExportDraftStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DataExportDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_data_export_draft_engine(flags: &flags::Registry) -> DataExportDraftEngine {
    DataExportDraftEngine {
        steps: vec![
            Rc::new(DataExportDraftThresholdOffsetAdjuster { weight: 1.59 }) as Rc<dyn DataExportDraftStep>,
            Rc::new(PercentDataExportDraftAdjuster { percent: 24.9 }),
            Rc::new(CapDataExportDraftAdjuster { cap: 9187.0 }),
            Rc::new(FlaggedDataExportDraftAdjuster {
                flags: flags.clone(),
                flag: "data-export-draft-fallback".to_string(),
                percent: 6.5,
            }),
            Rc::new(RoundingDataExportDraftAdjuster { step: 1.0 }),
            Rc::new(MinimumDataExportDraftAdjuster { minimum: 25.0 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 69398.3,
    }
}
