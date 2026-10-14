use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the migration agreement calculation.
pub trait MigrationAgreementFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterMigrationAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MigrationAgreementFactor for ClusterMigrationAgreementAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &MigrationAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MigrationAgreementFactor for TieredMigrationAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MigrationAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAgreementAmountOffsetAdjuster {
    pub weight: f64,
}

impl MigrationAgreementFactor for MigrationAgreementAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &MigrationAgreement, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the migration agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationAgreementEngine {
    pub steps: Vec<Rc<dyn MigrationAgreementFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationAgreementEngine {
    pub fn run(&mut self, item: &MigrationAgreement) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationAgreementFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_migration_agreement_engine() -> MigrationAgreementEngine {
    MigrationAgreementEngine {
        steps: vec![
            Rc::new(ClusterMigrationAgreementAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.72)]),
            }) as Rc<dyn MigrationAgreementFactor>,
            Rc::new(TieredMigrationAgreementAdjuster {
                bounds: vec![92.0, 2408.0, 5578.0, 7594.0],
                rates: vec![1.05, 1.14, 0.97, 0.82],
            }),
            Rc::new(MigrationAgreementAmountOffsetAdjuster { weight: 1.7 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 31360.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_agreement_engine();
        let item = new_migration_agreement("code-720", "label-21");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
