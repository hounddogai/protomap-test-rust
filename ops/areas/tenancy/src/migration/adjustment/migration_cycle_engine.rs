use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the migration cycle calculation.
pub trait MigrationCycleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationCycleAdjuster {
    pub minimum: f64,
}

impl MigrationCycleModifier for MinimumMigrationCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMigrationCycleAdjuster {
    pub step: f64,
}

impl MigrationCycleModifier for RoundingMigrationCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MigrationCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelMigrationCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MigrationCycleModifier for ChannelMigrationCycleAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &MigrationCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMigrationCycleAdjuster {
    pub percent: f64,
}

impl MigrationCycleModifier for PercentMigrationCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MigrationCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the migration cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationCycleEngine {
    pub steps: Vec<Rc<dyn MigrationCycleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationCycleEngine {
    pub fn compute(&mut self, item: &MigrationCycle) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[MigrationCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_migration_cycle_engine() -> MigrationCycleEngine {
    MigrationCycleEngine {
        steps: vec![
            Rc::new(MinimumMigrationCycleAdjuster { minimum: 26.4 }) as Rc<dyn MigrationCycleModifier>,
            Rc::new(RoundingMigrationCycleAdjuster { step: 0.05 }),
            Rc::new(ChannelMigrationCycleAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.91),
                    ("south".to_string(), 1.24),
                    ("east".to_string(), 0.82),
                ]),
            }),
            Rc::new(PercentMigrationCycleAdjuster { percent: 1.6 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 11896.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_cycle_engine();
        let item = new_migration_cycle("code-383", "label-103");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
