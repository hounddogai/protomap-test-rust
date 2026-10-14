use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the scheduled migration manifest calculation.
pub trait ScheduledMigrationManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledMigrationManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledMigrationManifestAdjuster {
    pub cap: f64,
}

impl ScheduledMigrationManifestAdjuster for CapScheduledMigrationManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledMigrationManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledMigrationManifestAdjuster {
    pub percent: f64,
}

impl ScheduledMigrationManifestAdjuster for PercentScheduledMigrationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledMigrationManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledMigrationManifestAdjuster {
    pub minimum: f64,
}

impl ScheduledMigrationManifestAdjuster for MinimumScheduledMigrationManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledMigrationManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledMigrationManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledMigrationManifestAdjuster for TieredScheduledMigrationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledMigrationManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMigrationManifestCapacityOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledMigrationManifestAdjuster for ScheduledMigrationManifestCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledMigrationManifest, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

/// Runs the scheduled migration manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledMigrationManifestEngine {
    pub steps: Vec<Rc<dyn ScheduledMigrationManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledMigrationManifestEngine {
    pub fn price(&mut self, item: &ScheduledMigrationManifest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledMigrationManifestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledMigrationManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_migration_manifest_engine() -> ScheduledMigrationManifestEngine {
    ScheduledMigrationManifestEngine {
        steps: vec![
            Rc::new(CapScheduledMigrationManifestAdjuster { cap: 39300.0 })
                as Rc<dyn ScheduledMigrationManifestAdjuster>,
            Rc::new(PercentScheduledMigrationManifestAdjuster { percent: -17.5 }),
            Rc::new(MinimumScheduledMigrationManifestAdjuster { minimum: 0.6 }),
            Rc::new(TieredScheduledMigrationManifestAdjuster {
                bounds: vec![2785.0, 4815.0],
                rates: vec![1.07, 1.03],
            }),
            Rc::new(ScheduledMigrationManifestCapacityOffsetAdjuster { weight: 0.61 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 89583.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_migration_manifest_engine();
        let item = new_scheduled_migration_manifest("code-68", "title-462");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
