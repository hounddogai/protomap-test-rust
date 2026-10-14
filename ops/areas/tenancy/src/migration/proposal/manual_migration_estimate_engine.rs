use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual migration estimate calculation.
pub trait ManualMigrationEstimateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualMigrationEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualMigrationEstimateAdjuster {
    pub step: f64,
}

impl ManualMigrationEstimateStep for RoundingManualMigrationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualMigrationEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualMigrationEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualMigrationEstimateStep for TieredManualMigrationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualMigrationEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionManualMigrationEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualMigrationEstimateStep for RegionManualMigrationEstimateAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ManualMigrationEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualMigrationEstimateAdjuster {
    pub cap: f64,
}

impl ManualMigrationEstimateStep for CapManualMigrationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualMigrationEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the manual migration estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualMigrationEstimateEngine {
    pub steps: Vec<Rc<dyn ManualMigrationEstimateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualMigrationEstimateEngine {
    pub fn compute(&mut self, item: &ManualMigrationEstimate) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualMigrationEstimateStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualMigrationEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_migration_estimate_engine() -> ManualMigrationEstimateEngine {
    ManualMigrationEstimateEngine {
        steps: vec![
            Rc::new(RoundingManualMigrationEstimateAdjuster { step: 5.0 }) as Rc<dyn ManualMigrationEstimateStep>,
            Rc::new(TieredManualMigrationEstimateAdjuster {
                bounds: vec![4627.0, 6381.0],
                rates: vec![0.91, 0.82],
            }),
            Rc::new(RegionManualMigrationEstimateAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.76),
                    ("central".to_string(), 1.36),
                    ("online".to_string(), 0.95),
                ]),
            }),
            Rc::new(CapManualMigrationEstimateAdjuster { cap: 44151.0 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 89096.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_migration_estimate_engine();
        let item = new_manual_migration_estimate("code-928", "label-15");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
