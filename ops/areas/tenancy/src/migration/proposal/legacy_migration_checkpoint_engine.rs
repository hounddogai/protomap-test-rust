use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy migration checkpoint calculation.
pub trait LegacyMigrationCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyMigrationCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationCheckpointWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl LegacyMigrationCheckpointModifier for LegacyMigrationCheckpointWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyMigrationCheckpoint, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeLegacyMigrationCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyMigrationCheckpointModifier for ScopeLegacyMigrationCheckpointAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &LegacyMigrationCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyMigrationCheckpointAdjuster {
    pub minimum: f64,
}

impl LegacyMigrationCheckpointModifier for MinimumLegacyMigrationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyMigrationCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyMigrationCheckpointAdjuster {
    pub percent: f64,
}

impl LegacyMigrationCheckpointModifier for PercentLegacyMigrationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyMigrationCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyMigrationCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyMigrationCheckpointModifier for TieredLegacyMigrationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyMigrationCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the legacy migration checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyMigrationCheckpointEngine {
    pub steps: Vec<Rc<dyn LegacyMigrationCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyMigrationCheckpointEngine {
    pub fn compute(&mut self, item: &LegacyMigrationCheckpoint) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[LegacyMigrationCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_migration_checkpoint_engine() -> LegacyMigrationCheckpointEngine {
    LegacyMigrationCheckpointEngine {
        steps: vec![
            Rc::new(LegacyMigrationCheckpointWindowDaysOffsetAdjuster { weight: 0.9 })
                as Rc<dyn LegacyMigrationCheckpointModifier>,
            Rc::new(ScopeLegacyMigrationCheckpointAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.28),
                    ("online".to_string(), 1.33),
                    ("retail".to_string(), 0.98),
                ]),
            }),
            Rc::new(MinimumLegacyMigrationCheckpointAdjuster { minimum: 22.0 }),
            Rc::new(PercentLegacyMigrationCheckpointAdjuster { percent: 5.6 }),
            Rc::new(TieredLegacyMigrationCheckpointAdjuster {
                bounds: vec![2708.0, 8268.0, 8287.0],
                rates: vec![0.89, 0.92, 1.09],
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 47431.6,
    }
}
