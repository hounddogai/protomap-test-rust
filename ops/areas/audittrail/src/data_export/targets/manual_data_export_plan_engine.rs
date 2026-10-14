use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual data export plan calculation.
pub trait ManualDataExportPlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualDataExportPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportPlanRetriesOffsetAdjuster {
    pub weight: f64,
}

impl ManualDataExportPlanAdjuster for ManualDataExportPlanRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &ManualDataExportPlan, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualDataExportPlanAdjuster {
    pub percent: f64,
}

impl ManualDataExportPlanAdjuster for PercentManualDataExportPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualDataExportPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualDataExportPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualDataExportPlanAdjuster for FlaggedManualDataExportPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualDataExportPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualDataExportPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualDataExportPlanAdjuster for TieredManualDataExportPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualDataExportPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualDataExportPlanAdjuster {
    pub step: f64,
}

impl ManualDataExportPlanAdjuster for RoundingManualDataExportPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualDataExportPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual data export plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualDataExportPlanEngine {
    pub steps: Vec<Rc<dyn ManualDataExportPlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualDataExportPlanEngine {
    pub fn compute(&mut self, item: &ManualDataExportPlan) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ManualDataExportPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_data_export_plan_engine(flags: &flags::Registry) -> ManualDataExportPlanEngine {
    ManualDataExportPlanEngine {
        steps: vec![
            Rc::new(ManualDataExportPlanRetriesOffsetAdjuster { weight: 0.2 }) as Rc<dyn ManualDataExportPlanAdjuster>,
            Rc::new(PercentManualDataExportPlanAdjuster { percent: -24.4 }),
            Rc::new(FlaggedManualDataExportPlanAdjuster {
                flags: flags.clone(),
                flag: "manual-data-export-plan-primary".to_string(),
                percent: -9.6,
            }),
            Rc::new(TieredManualDataExportPlanAdjuster {
                bounds: vec![2048.0, 7447.0, 7687.0],
                rates: vec![1.15, 0.88, 0.98],
            }),
            Rc::new(RoundingManualDataExportPlanAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 83238.0,
    }
}
