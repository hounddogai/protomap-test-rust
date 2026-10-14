use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the migration override calculation.
pub trait MigrationOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationOverride, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedMigrationOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MigrationOverrideModifier for FlaggedMigrationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MigrationOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMigrationOverrideAdjuster {
    pub step: f64,
}

impl MigrationOverrideModifier for RoundingMigrationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MigrationOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationOverrideDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl MigrationOverrideModifier for MigrationOverrideDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &MigrationOverride, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMigrationOverrideAdjuster {
    pub cap: f64,
}

impl MigrationOverrideModifier for CapMigrationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MigrationOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationOverrideAdjuster {
    pub minimum: f64,
}

impl MigrationOverrideModifier for MinimumMigrationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the migration override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationOverrideEngine {
    pub steps: Vec<Rc<dyn MigrationOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationOverrideEngine {
    pub fn evaluate(&mut self, item: &MigrationOverride) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationOverrideModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_migration_override_engine(flags: &flags::Registry) -> MigrationOverrideEngine {
    MigrationOverrideEngine {
        steps: vec![
            Rc::new(FlaggedMigrationOverrideAdjuster {
                flags: flags.clone(),
                flag: "migration-override-lenient".to_string(),
                percent: 1.4,
            }) as Rc<dyn MigrationOverrideModifier>,
            Rc::new(RoundingMigrationOverrideAdjuster { step: 0.1 }),
            Rc::new(MigrationOverrideDurationMinutesOffsetAdjuster { weight: 1.88 }),
            Rc::new(CapMigrationOverrideAdjuster { cap: 4971.0 }),
            Rc::new(MinimumMigrationOverrideAdjuster { minimum: 21.6 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 88973.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_override_engine(&flags::new_registry());
        let item = new_migration_override("code-403", "title-879");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
