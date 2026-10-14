use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending migration schedule calculation.
pub trait PendingMigrationScheduleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingMigrationSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingMigrationScheduleAdjuster {
    pub minimum: f64,
}

impl PendingMigrationScheduleFactor for MinimumPendingMigrationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingMigrationSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingMigrationScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingMigrationScheduleFactor for FlaggedPendingMigrationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingMigrationSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingMigrationScheduleAdjuster {
    pub cap: f64,
}

impl PendingMigrationScheduleFactor for CapPendingMigrationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingMigrationSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingMigrationScheduleAdjuster {
    pub step: f64,
}

impl PendingMigrationScheduleFactor for RoundingPendingMigrationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingMigrationSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the pending migration schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingMigrationScheduleEngine {
    pub steps: Vec<Rc<dyn PendingMigrationScheduleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingMigrationScheduleEngine {
    pub fn evaluate(&mut self, item: &PendingMigrationSchedule) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[PendingMigrationSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_migration_schedule_engine(flags: &flags::Registry) -> PendingMigrationScheduleEngine {
    PendingMigrationScheduleEngine {
        steps: vec![
            Rc::new(MinimumPendingMigrationScheduleAdjuster { minimum: 30.7 })
                as Rc<dyn PendingMigrationScheduleFactor>,
            Rc::new(FlaggedPendingMigrationScheduleAdjuster {
                flags: flags.clone(),
                flag: "pending-migration-schedule-pending".to_string(),
                percent: -5.4,
            }),
            Rc::new(CapPendingMigrationScheduleAdjuster { cap: 36913.0 }),
            Rc::new(RoundingPendingMigrationScheduleAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 95854.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_migration_schedule_engine(&flags::new_registry());
        let item = new_pending_migration_schedule("code-222", "name-65");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
