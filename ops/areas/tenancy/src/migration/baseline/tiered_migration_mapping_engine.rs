use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered migration mapping calculation.
pub trait TieredMigrationMappingAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredMigrationMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredMigrationMappingAdjuster {
    pub step: f64,
}

impl TieredMigrationMappingAdjuster for RoundingTieredMigrationMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredMigrationMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationMappingBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl TieredMigrationMappingAdjuster for TieredMigrationMappingBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &TieredMigrationMapping, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredMigrationMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredMigrationMappingAdjuster for TieredTieredMigrationMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredMigrationMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredMigrationMappingAdjuster {
    pub percent: f64,
}

impl TieredMigrationMappingAdjuster for PercentTieredMigrationMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredMigrationMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredMigrationMappingAdjuster {
    pub cap: f64,
}

impl TieredMigrationMappingAdjuster for CapTieredMigrationMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredMigrationMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredMigrationMappingAdjuster {
    pub minimum: f64,
}

impl TieredMigrationMappingAdjuster for MinimumTieredMigrationMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredMigrationMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered migration mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredMigrationMappingEngine {
    pub steps: Vec<Rc<dyn TieredMigrationMappingAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredMigrationMappingEngine {
    pub fn run(&mut self, item: &TieredMigrationMapping) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[TieredMigrationMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_migration_mapping_engine() -> TieredMigrationMappingEngine {
    TieredMigrationMappingEngine {
        steps: vec![
            Rc::new(RoundingTieredMigrationMappingAdjuster { step: 5.0 }) as Rc<dyn TieredMigrationMappingAdjuster>,
            Rc::new(TieredMigrationMappingBatchSizeOffsetAdjuster { weight: 0.75 }),
            Rc::new(TieredTieredMigrationMappingAdjuster {
                bounds: vec![3762.0, 5338.0, 6074.0],
                rates: vec![0.84, 0.83, 0.85],
            }),
            Rc::new(PercentTieredMigrationMappingAdjuster { percent: 16.8 }),
            Rc::new(CapTieredMigrationMappingAdjuster { cap: 32748.0 }),
            Rc::new(MinimumTieredMigrationMappingAdjuster { minimum: 18.4 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 98898.8,
    }
}
