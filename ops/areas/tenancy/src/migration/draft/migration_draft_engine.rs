use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the migration draft calculation.
pub trait MigrationDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationDraftQuantityOffsetAdjuster {
    pub weight: f64,
}

impl MigrationDraftFactor for MigrationDraftQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &MigrationDraft, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMigrationDraftAdjuster {
    pub cap: f64,
}

impl MigrationDraftFactor for CapMigrationDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MigrationDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMigrationDraftAdjuster {
    pub step: f64,
}

impl MigrationDraftFactor for RoundingMigrationDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MigrationDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MigrationDraftFactor for TieredMigrationDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MigrationDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMigrationDraftAdjuster {
    pub percent: f64,
}

impl MigrationDraftFactor for PercentMigrationDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MigrationDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryMigrationDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MigrationDraftFactor for CategoryMigrationDraftAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &MigrationDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the migration draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationDraftEngine {
    pub steps: Vec<Rc<dyn MigrationDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationDraftEngine {
    pub fn compute(&mut self, item: &MigrationDraft) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationDraftFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_migration_draft_engine() -> MigrationDraftEngine {
    MigrationDraftEngine {
        steps: vec![
            Rc::new(MigrationDraftQuantityOffsetAdjuster { weight: 1.61 }) as Rc<dyn MigrationDraftFactor>,
            Rc::new(CapMigrationDraftAdjuster { cap: 22041.0 }),
            Rc::new(RoundingMigrationDraftAdjuster { step: 0.5 }),
            Rc::new(TieredMigrationDraftAdjuster {
                bounds: vec![5478.0, 8065.0],
                rates: vec![1.09, 0.91],
            }),
            Rc::new(PercentMigrationDraftAdjuster { percent: -3.8 }),
            Rc::new(CategoryMigrationDraftAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.76)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 29394.5,
    }
}
