use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the data export plan calculation.
pub trait DataExportPlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportPlanAdjuster {
    pub percent: f64,
}

impl DataExportPlanFactor for PercentDataExportPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportPlanAdjuster {
    pub cap: f64,
}

impl DataExportPlanFactor for CapDataExportPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DataExportPlanFactor for TieredDataExportPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DataExportPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the data export plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportPlanEngine {
    pub steps: Vec<Rc<dyn DataExportPlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportPlanEngine {
    pub fn evaluate(&mut self, item: &DataExportPlan) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[DataExportPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_data_export_plan_engine() -> DataExportPlanEngine {
    DataExportPlanEngine {
        steps: vec![
            Rc::new(PercentDataExportPlanAdjuster { percent: -20.1 }) as Rc<dyn DataExportPlanFactor>,
            Rc::new(CapDataExportPlanAdjuster { cap: 20157.0 }),
            Rc::new(TieredDataExportPlanAdjuster {
                bounds: vec![1782.0, 6474.0, 8783.0],
                rates: vec![1.07, 1.05, 1.02],
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 69017.2,
    }
}
