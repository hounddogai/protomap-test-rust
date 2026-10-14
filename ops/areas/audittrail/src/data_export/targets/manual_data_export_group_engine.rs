use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the manual data export group calculation.
pub trait ManualDataExportGroupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualDataExportGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualDataExportGroupAdjuster {
    pub percent: f64,
}

impl ManualDataExportGroupFactor for PercentManualDataExportGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualDataExportGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualDataExportGroupAdjuster {
    pub minimum: f64,
}

impl ManualDataExportGroupFactor for MinimumManualDataExportGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualDataExportGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionManualDataExportGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualDataExportGroupFactor for RegionManualDataExportGroupAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ManualDataExportGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the manual data export group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualDataExportGroupEngine {
    pub steps: Vec<Rc<dyn ManualDataExportGroupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualDataExportGroupEngine {
    pub fn run(&mut self, item: &ManualDataExportGroup) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualDataExportGroupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualDataExportGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_data_export_group_engine() -> ManualDataExportGroupEngine {
    ManualDataExportGroupEngine {
        steps: vec![
            Rc::new(PercentManualDataExportGroupAdjuster { percent: 1.0 }) as Rc<dyn ManualDataExportGroupFactor>,
            Rc::new(MinimumManualDataExportGroupAdjuster { minimum: 46.4 }),
            Rc::new(RegionManualDataExportGroupAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.75),
                    ("east".to_string(), 0.86),
                    ("south".to_string(), 0.92),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 17580.2,
    }
}
