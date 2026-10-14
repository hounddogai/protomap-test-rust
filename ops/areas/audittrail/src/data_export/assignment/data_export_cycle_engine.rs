use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the data export cycle calculation.
pub trait DataExportCycleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeDataExportCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportCycleFactor for ScopeDataExportCycleAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &DataExportCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportCycleFactor for FlaggedDataExportCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportCycleCeilingOffsetAdjuster {
    pub weight: f64,
}

impl DataExportCycleFactor for DataExportCycleCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &DataExportCycle, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportCycleAdjuster {
    pub cap: f64,
}

impl DataExportCycleFactor for CapDataExportCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the data export cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportCycleEngine {
    pub steps: Vec<Rc<dyn DataExportCycleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportCycleEngine {
    pub fn compute(&mut self, item: &DataExportCycle) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DataExportCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_data_export_cycle_engine(flags: &flags::Registry) -> DataExportCycleEngine {
    DataExportCycleEngine {
        steps: vec![
            Rc::new(ScopeDataExportCycleAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.74), ("central".to_string(), 0.98)]),
            }) as Rc<dyn DataExportCycleFactor>,
            Rc::new(FlaggedDataExportCycleAdjuster {
                flags: flags.clone(),
                flag: "data-export-cycle-tiered".to_string(),
                percent: -5.2,
            }),
            Rc::new(DataExportCycleCeilingOffsetAdjuster { weight: 0.95 }),
            Rc::new(CapDataExportCycleAdjuster { cap: 2959.0 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 65022.1,
    }
}
