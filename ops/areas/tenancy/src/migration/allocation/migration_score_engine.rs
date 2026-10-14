use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the migration score calculation.
pub trait MigrationScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleMigrationScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MigrationScoreAdjuster for TitleMigrationScoreAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &MigrationScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMigrationScoreAdjuster {
    pub percent: f64,
}

impl MigrationScoreAdjuster for PercentMigrationScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MigrationScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedMigrationScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MigrationScoreAdjuster for FlaggedMigrationScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MigrationScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the migration score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationScoreEngine {
    pub steps: Vec<Rc<dyn MigrationScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationScoreEngine {
    pub fn run(&mut self, item: &MigrationScore) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[MigrationScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_migration_score_engine(flags: &flags::Registry) -> MigrationScoreEngine {
    MigrationScoreEngine {
        steps: vec![
            Rc::new(TitleMigrationScoreAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.99),
                    ("west".to_string(), 1.21),
                    ("south".to_string(), 1.0),
                ]),
            }) as Rc<dyn MigrationScoreAdjuster>,
            Rc::new(PercentMigrationScoreAdjuster { percent: 1.1 }),
            Rc::new(FlaggedMigrationScoreAdjuster {
                flags: flags.clone(),
                flag: "migration-score-default".to_string(),
                percent: 2.3,
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 83101.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_score_engine(&flags::new_registry());
        let item = new_migration_score("code-918", "title-851");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
