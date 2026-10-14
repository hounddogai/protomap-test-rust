use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the scheduled dashboard record calculation.
pub trait ScheduledDashboardRecordModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledDashboardRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledDashboardRecordAdjuster {
    pub cap: f64,
}

impl ScheduledDashboardRecordModifier for CapScheduledDashboardRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledDashboardRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledDashboardRecordAdjuster {
    pub percent: f64,
}

impl ScheduledDashboardRecordModifier for PercentScheduledDashboardRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledDashboardRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledDashboardRecordAdjuster {
    pub minimum: f64,
}

impl ScheduledDashboardRecordModifier for MinimumScheduledDashboardRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledDashboardRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the scheduled dashboard record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledDashboardRecordEngine {
    pub steps: Vec<Rc<dyn ScheduledDashboardRecordModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledDashboardRecordEngine {
    pub fn compute(&mut self, item: &ScheduledDashboardRecord) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledDashboardRecordModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledDashboardRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_dashboard_record_engine() -> ScheduledDashboardRecordEngine {
    ScheduledDashboardRecordEngine {
        steps: vec![
            Rc::new(CapScheduledDashboardRecordAdjuster { cap: 26706.0 }) as Rc<dyn ScheduledDashboardRecordModifier>,
            Rc::new(PercentScheduledDashboardRecordAdjuster { percent: -14.9 }),
            Rc::new(MinimumScheduledDashboardRecordAdjuster { minimum: 34.4 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 96206.2,
    }
}
