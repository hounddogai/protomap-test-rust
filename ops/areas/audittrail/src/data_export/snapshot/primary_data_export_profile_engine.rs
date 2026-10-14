use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary data export profile calculation.
pub trait PrimaryDataExportProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryDataExportProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDataExportProfileStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryDataExportProfileAdjuster for PrimaryDataExportProfileStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryDataExportProfile, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryDataExportProfileAdjuster {
    pub percent: f64,
}

impl PrimaryDataExportProfileAdjuster for PercentPrimaryDataExportProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryDataExportProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryDataExportProfileAdjuster {
    pub step: f64,
}

impl PrimaryDataExportProfileAdjuster for RoundingPrimaryDataExportProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryDataExportProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the primary data export profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryDataExportProfileEngine {
    pub steps: Vec<Rc<dyn PrimaryDataExportProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryDataExportProfileEngine {
    pub fn run(&mut self, item: &PrimaryDataExportProfile) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryDataExportProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_data_export_profile_engine() -> PrimaryDataExportProfileEngine {
    PrimaryDataExportProfileEngine {
        steps: vec![
            Rc::new(PrimaryDataExportProfileStartsAtOffsetAdjuster { weight: 0.04 })
                as Rc<dyn PrimaryDataExportProfileAdjuster>,
            Rc::new(PercentPrimaryDataExportProfileAdjuster { percent: -14.0 }),
            Rc::new(RoundingPrimaryDataExportProfileAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 84672.7,
    }
}
