use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the rolling migration agreement calculation.
pub trait RollingMigrationAgreementAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingMigrationAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingMigrationAgreementAdjuster {
    pub minimum: f64,
}

impl RollingMigrationAgreementAdjuster for MinimumRollingMigrationAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingMigrationAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingMigrationAgreementAdjuster {
    pub percent: f64,
}

impl RollingMigrationAgreementAdjuster for PercentRollingMigrationAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingMigrationAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationAgreementCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl RollingMigrationAgreementAdjuster for RollingMigrationAgreementCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &RollingMigrationAgreement, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRollingMigrationAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingMigrationAgreementAdjuster for TitleRollingMigrationAgreementAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RollingMigrationAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rolling migration agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingMigrationAgreementEngine {
    pub steps: Vec<Rc<dyn RollingMigrationAgreementAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingMigrationAgreementEngine {
    pub fn compute(&mut self, item: &RollingMigrationAgreement) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingMigrationAgreementAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingMigrationAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_migration_agreement_engine() -> RollingMigrationAgreementEngine {
    RollingMigrationAgreementEngine {
        steps: vec![
            Rc::new(MinimumRollingMigrationAgreementAdjuster { minimum: 25.6 })
                as Rc<dyn RollingMigrationAgreementAdjuster>,
            Rc::new(PercentRollingMigrationAgreementAdjuster { percent: -5.9 }),
            Rc::new(RollingMigrationAgreementCreatedAtOffsetAdjuster { weight: 0.39 }),
            Rc::new(TitleRollingMigrationAgreementAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.74)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 89810.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_migration_agreement_engine();
        let item = new_rolling_migration_agreement("code-136", "title-346");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
