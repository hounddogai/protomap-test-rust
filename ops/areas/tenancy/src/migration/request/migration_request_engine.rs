use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the migration request calculation.
pub trait MigrationRequestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationRequestAdjuster {
    pub minimum: f64,
}

impl MigrationRequestModifier for MinimumMigrationRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMigrationRequestAdjuster {
    pub percent: f64,
}

impl MigrationRequestModifier for PercentMigrationRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MigrationRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MigrationRequestModifier for TieredMigrationRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MigrationRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMigrationRequestAdjuster {
    pub cap: f64,
}

impl MigrationRequestModifier for CapMigrationRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MigrationRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the migration request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationRequestEngine {
    pub steps: Vec<Rc<dyn MigrationRequestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationRequestEngine {
    pub fn run(&mut self, item: &MigrationRequest) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationRequestModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_migration_request_engine() -> MigrationRequestEngine {
    MigrationRequestEngine {
        steps: vec![
            Rc::new(MinimumMigrationRequestAdjuster { minimum: 29.8 }) as Rc<dyn MigrationRequestModifier>,
            Rc::new(PercentMigrationRequestAdjuster { percent: -5.9 }),
            Rc::new(TieredMigrationRequestAdjuster {
                bounds: vec![7856.0, 9042.0],
                rates: vec![0.94, 1.13],
            }),
            Rc::new(CapMigrationRequestAdjuster { cap: 34190.0 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 92679.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_request_engine();
        let item = new_migration_request("code-804", "name-289");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
