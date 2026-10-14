use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending migration slot calculation.
pub trait PendingMigrationSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingMigrationSlot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingMigrationSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingMigrationSlotStep for FlaggedPendingMigrationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingMigrationSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingMigrationSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingMigrationSlotStep for TieredPendingMigrationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingMigrationSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePendingMigrationSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingMigrationSlotStep for SourcePendingMigrationSlotAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PendingMigrationSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the pending migration slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingMigrationSlotEngine {
    pub steps: Vec<Rc<dyn PendingMigrationSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingMigrationSlotEngine {
    pub fn compute(&mut self, item: &PendingMigrationSlot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PendingMigrationSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_migration_slot_engine(flags: &flags::Registry) -> PendingMigrationSlotEngine {
    PendingMigrationSlotEngine {
        steps: vec![
            Rc::new(FlaggedPendingMigrationSlotAdjuster {
                flags: flags.clone(),
                flag: "pending-migration-slot-rolling".to_string(),
                percent: 2.1,
            }) as Rc<dyn PendingMigrationSlotStep>,
            Rc::new(TieredPendingMigrationSlotAdjuster {
                bounds: vec![1535.0, 3207.0, 7307.0],
                rates: vec![0.86, 0.86, 1.13],
            }),
            Rc::new(SourcePendingMigrationSlotAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.25)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 83962.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_migration_slot_engine(&flags::new_registry());
        let item = new_pending_migration_slot("code-841", "title-208");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
