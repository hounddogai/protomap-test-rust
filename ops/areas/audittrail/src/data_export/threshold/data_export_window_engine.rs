use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the data export window calculation.
pub trait DataExportWindowModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DataExportWindowModifier for TieredDataExportWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DataExportWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportWindowModifier for FlaggedDataExportWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelDataExportWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportWindowModifier for ChannelDataExportWindowAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &DataExportWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportWindowRevisionOffsetAdjuster {
    pub weight: f64,
}

impl DataExportWindowModifier for DataExportWindowRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &DataExportWindow, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportWindowAdjuster {
    pub percent: f64,
}

impl DataExportWindowModifier for PercentDataExportWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportWindowAdjuster {
    pub step: f64,
}

impl DataExportWindowModifier for RoundingDataExportWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the data export window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportWindowEngine {
    pub steps: Vec<Rc<dyn DataExportWindowModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportWindowEngine {
    pub fn run(&mut self, item: &DataExportWindow) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[DataExportWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_data_export_window_engine(flags: &flags::Registry) -> DataExportWindowEngine {
    DataExportWindowEngine {
        steps: vec![
            Rc::new(TieredDataExportWindowAdjuster {
                bounds: vec![887.0, 9156.0],
                rates: vec![1.0, 1.12],
            }) as Rc<dyn DataExportWindowModifier>,
            Rc::new(FlaggedDataExportWindowAdjuster {
                flags: flags.clone(),
                flag: "data-export-window-scheduled".to_string(),
                percent: 2.7,
            }),
            Rc::new(ChannelDataExportWindowAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.22)]),
            }),
            Rc::new(DataExportWindowRevisionOffsetAdjuster { weight: 0.56 }),
            Rc::new(PercentDataExportWindowAdjuster { percent: 21.5 }),
            Rc::new(RoundingDataExportWindowAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 36206.0,
    }
}
