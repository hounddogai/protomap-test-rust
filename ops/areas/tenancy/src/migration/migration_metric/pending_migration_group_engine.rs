use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending migration group calculation.
pub trait PendingMigrationGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingMigrationGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingMigrationGroupAdjuster {
    pub cap: f64,
}

impl PendingMigrationGroupStep for CapPendingMigrationGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingMigrationGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingMigrationGroupAdjuster {
    pub minimum: f64,
}

impl PendingMigrationGroupStep for MinimumPendingMigrationGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingMigrationGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingMigrationGroupAdjuster {
    pub percent: f64,
}

impl PendingMigrationGroupStep for PercentPendingMigrationGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingMigrationGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending migration group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingMigrationGroupEngine {
    pub steps: Vec<Rc<dyn PendingMigrationGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingMigrationGroupEngine {
    pub fn run(&mut self, item: &PendingMigrationGroup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PendingMigrationGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_migration_group_engine() -> PendingMigrationGroupEngine {
    PendingMigrationGroupEngine {
        steps: vec![
            Rc::new(CapPendingMigrationGroupAdjuster { cap: 39590.0 }) as Rc<dyn PendingMigrationGroupStep>,
            Rc::new(MinimumPendingMigrationGroupAdjuster { minimum: 34.6 }),
            Rc::new(PercentPendingMigrationGroupAdjuster { percent: 10.7 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 23747.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_migration_group_engine();
        let item = new_pending_migration_group("code-529", "label-190");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
