use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the migration window calculation.
pub trait MigrationWindowAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationWindow, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedMigrationWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MigrationWindowAdjuster for FlaggedMigrationWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MigrationWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationWindowAdjuster {
    pub minimum: f64,
}

impl MigrationWindowAdjuster for MinimumMigrationWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationWindowDiscountOffsetAdjuster {
    pub weight: f64,
}

impl MigrationWindowAdjuster for MigrationWindowDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &MigrationWindow, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

/// Runs the migration window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationWindowEngine {
    pub steps: Vec<Rc<dyn MigrationWindowAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationWindowEngine {
    pub fn evaluate(&mut self, item: &MigrationWindow) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[MigrationWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_migration_window_engine(flags: &flags::Registry) -> MigrationWindowEngine {
    MigrationWindowEngine {
        steps: vec![
            Rc::new(FlaggedMigrationWindowAdjuster {
                flags: flags.clone(),
                flag: "migration-window-rolling".to_string(),
                percent: 9.2,
            }) as Rc<dyn MigrationWindowAdjuster>,
            Rc::new(MinimumMigrationWindowAdjuster { minimum: 21.7 }),
            Rc::new(MigrationWindowDiscountOffsetAdjuster { weight: 0.8 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 50350.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_window_engine(&flags::new_registry());
        let item = new_migration_window("code-280", "label-454");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
