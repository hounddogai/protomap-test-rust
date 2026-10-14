use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the migration schedule calculation.
pub trait MigrationScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationScheduleAdjuster {
    pub minimum: f64,
}

impl MigrationScheduleStep for MinimumMigrationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMigrationScheduleAdjuster {
    pub step: f64,
}

impl MigrationScheduleStep for RoundingMigrationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MigrationSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMigrationScheduleAdjuster {
    pub percent: f64,
}

impl MigrationScheduleStep for PercentMigrationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MigrationSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MigrationScheduleStep for TieredMigrationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MigrationSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMigrationScheduleAdjuster {
    pub cap: f64,
}

impl MigrationScheduleStep for CapMigrationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MigrationSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the migration schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationScheduleEngine {
    pub steps: Vec<Rc<dyn MigrationScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationScheduleEngine {
    pub fn run(&mut self, item: &MigrationSchedule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[MigrationSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_migration_schedule_engine() -> MigrationScheduleEngine {
    MigrationScheduleEngine {
        steps: vec![
            Rc::new(MinimumMigrationScheduleAdjuster { minimum: 26.8 }) as Rc<dyn MigrationScheduleStep>,
            Rc::new(RoundingMigrationScheduleAdjuster { step: 0.25 }),
            Rc::new(PercentMigrationScheduleAdjuster { percent: 3.4 }),
            Rc::new(TieredMigrationScheduleAdjuster {
                bounds: vec![3193.0, 6101.0],
                rates: vec![1.02, 1.04],
            }),
            Rc::new(CapMigrationScheduleAdjuster { cap: 5243.0 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 44305.8,
    }
}
