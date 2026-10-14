use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the migration threshold calculation.
pub trait MigrationThresholdFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMigrationThresholdAdjuster {
    pub step: f64,
}

impl MigrationThresholdFactor for RoundingMigrationThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MigrationThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedMigrationThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MigrationThresholdFactor for FlaggedMigrationThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MigrationThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeMigrationThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MigrationThresholdFactor for ScopeMigrationThresholdAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &MigrationThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the migration threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationThresholdEngine {
    pub steps: Vec<Rc<dyn MigrationThresholdFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationThresholdEngine {
    pub fn compute(&mut self, item: &MigrationThreshold) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationThresholdFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_migration_threshold_engine(flags: &flags::Registry) -> MigrationThresholdEngine {
    MigrationThresholdEngine {
        steps: vec![
            Rc::new(RoundingMigrationThresholdAdjuster { step: 0.25 }) as Rc<dyn MigrationThresholdFactor>,
            Rc::new(FlaggedMigrationThresholdAdjuster {
                flags: flags.clone(),
                flag: "migration-threshold-scheduled".to_string(),
                percent: -6.1,
            }),
            Rc::new(ScopeMigrationThresholdAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.13), ("online".to_string(), 0.94)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 42247.9,
    }
}
