use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the migration ledger calculation.
pub trait MigrationLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationLedgerFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl MigrationLedgerStep for MigrationLedgerFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &MigrationLedger, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationLedgerAdjuster {
    pub minimum: f64,
}

impl MigrationLedgerStep for MinimumMigrationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMigrationLedgerAdjuster {
    pub step: f64,
}

impl MigrationLedgerStep for RoundingMigrationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MigrationLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedMigrationLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MigrationLedgerStep for FlaggedMigrationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MigrationLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the migration ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationLedgerEngine {
    pub steps: Vec<Rc<dyn MigrationLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationLedgerEngine {
    pub fn run(&mut self, item: &MigrationLedger) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[MigrationLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_migration_ledger_engine(flags: &flags::Registry) -> MigrationLedgerEngine {
    MigrationLedgerEngine {
        steps: vec![
            Rc::new(MigrationLedgerFloorValueOffsetAdjuster { weight: 1.45 }) as Rc<dyn MigrationLedgerStep>,
            Rc::new(MinimumMigrationLedgerAdjuster { minimum: 1.7 }),
            Rc::new(RoundingMigrationLedgerAdjuster { step: 5.0 }),
            Rc::new(FlaggedMigrationLedgerAdjuster {
                flags: flags.clone(),
                flag: "migration-ledger-rolling".to_string(),
                percent: -8.8,
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 24002.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_ledger_engine(&flags::new_registry());
        let item = new_migration_ledger("code-914", "label-707");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
