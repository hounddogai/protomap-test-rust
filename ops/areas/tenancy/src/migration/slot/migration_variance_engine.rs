use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the migration variance calculation.
pub trait MigrationVarianceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MigrationVarianceFactor for TieredMigrationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MigrationVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMigrationVarianceAdjuster {
    pub cap: f64,
}

impl MigrationVarianceFactor for CapMigrationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MigrationVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMigrationVarianceAdjuster {
    pub percent: f64,
}

impl MigrationVarianceFactor for PercentMigrationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MigrationVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the migration variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationVarianceEngine {
    pub steps: Vec<Rc<dyn MigrationVarianceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationVarianceEngine {
    pub fn price(&mut self, item: &MigrationVariance) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationVarianceFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_migration_variance_engine() -> MigrationVarianceEngine {
    MigrationVarianceEngine {
        steps: vec![
            Rc::new(TieredMigrationVarianceAdjuster {
                bounds: vec![5555.0, 9576.0],
                rates: vec![0.96, 0.89],
            }) as Rc<dyn MigrationVarianceFactor>,
            Rc::new(CapMigrationVarianceAdjuster { cap: 9553.0 }),
            Rc::new(PercentMigrationVarianceAdjuster { percent: -4.4 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 48895.3,
    }
}
