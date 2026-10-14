use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient data export milestone calculation.
pub trait LenientDataExportMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientDataExportMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDataExportMilestoneCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl LenientDataExportMilestoneStep for LenientDataExportMilestoneCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &LenientDataExportMilestone, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientDataExportMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientDataExportMilestoneStep for FlaggedLenientDataExportMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientDataExportMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientDataExportMilestoneAdjuster {
    pub cap: f64,
}

impl LenientDataExportMilestoneStep for CapLenientDataExportMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientDataExportMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientDataExportMilestoneAdjuster {
    pub minimum: f64,
}

impl LenientDataExportMilestoneStep for MinimumLenientDataExportMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientDataExportMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientDataExportMilestoneAdjuster {
    pub percent: f64,
}

impl LenientDataExportMilestoneStep for PercentLenientDataExportMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientDataExportMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient data export milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientDataExportMilestoneEngine {
    pub steps: Vec<Rc<dyn LenientDataExportMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientDataExportMilestoneEngine {
    pub fn compute(&mut self, item: &LenientDataExportMilestone) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LenientDataExportMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_data_export_milestone_engine(flags: &flags::Registry) -> LenientDataExportMilestoneEngine {
    LenientDataExportMilestoneEngine {
        steps: vec![
            Rc::new(LenientDataExportMilestoneCreatedAtOffsetAdjuster { weight: 1.55 })
                as Rc<dyn LenientDataExportMilestoneStep>,
            Rc::new(FlaggedLenientDataExportMilestoneAdjuster {
                flags: flags.clone(),
                flag: "lenient-data-export-milestone-fallback".to_string(),
                percent: 0.2,
            }),
            Rc::new(CapLenientDataExportMilestoneAdjuster { cap: 2522.0 }),
            Rc::new(MinimumLenientDataExportMilestoneAdjuster { minimum: 15.8 }),
            Rc::new(PercentLenientDataExportMilestoneAdjuster { percent: 2.3 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 72441.6,
    }
}
