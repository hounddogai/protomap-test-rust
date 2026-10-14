use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the migration policy calculation.
pub trait MigrationPolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMigrationPolicyAdjuster {
    pub percent: f64,
}

impl MigrationPolicyModifier for PercentMigrationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MigrationPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMigrationPolicyAdjuster {
    pub cap: f64,
}

impl MigrationPolicyModifier for CapMigrationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MigrationPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationPolicyAdjuster {
    pub minimum: f64,
}

impl MigrationPolicyModifier for MinimumMigrationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MigrationPolicyModifier for TieredMigrationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MigrationPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationPolicyAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl MigrationPolicyModifier for MigrationPolicyAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &MigrationPolicy, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

/// Runs the migration policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationPolicyEngine {
    pub steps: Vec<Rc<dyn MigrationPolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationPolicyEngine {
    pub fn compute(&mut self, item: &MigrationPolicy) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationPolicyModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_migration_policy_engine() -> MigrationPolicyEngine {
    MigrationPolicyEngine {
        steps: vec![
            Rc::new(PercentMigrationPolicyAdjuster { percent: -4.8 }) as Rc<dyn MigrationPolicyModifier>,
            Rc::new(CapMigrationPolicyAdjuster { cap: 8076.0 }),
            Rc::new(MinimumMigrationPolicyAdjuster { minimum: 4.6 }),
            Rc::new(TieredMigrationPolicyAdjuster {
                bounds: vec![5404.0, 7218.0, 7311.0, 8366.0],
                rates: vec![1.05, 0.92, 0.96, 0.92],
            }),
            Rc::new(MigrationPolicyAttemptsOffsetAdjuster { weight: 1.19 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 3414.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_policy_engine();
        let item = new_migration_policy("code-195", "label-311");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
