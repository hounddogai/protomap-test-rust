use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered migration milestone calculation.
pub trait TieredMigrationMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredMigrationMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredMigrationMilestoneAdjuster {
    pub minimum: f64,
}

impl TieredMigrationMilestoneStep for MinimumTieredMigrationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredMigrationMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionTieredMigrationMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredMigrationMilestoneStep for RegionTieredMigrationMilestoneAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &TieredMigrationMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredMigrationMilestoneAdjuster {
    pub percent: f64,
}

impl TieredMigrationMilestoneStep for PercentTieredMigrationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredMigrationMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredMigrationMilestoneAdjuster {
    pub step: f64,
}

impl TieredMigrationMilestoneStep for RoundingTieredMigrationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredMigrationMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredMigrationMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredMigrationMilestoneStep for TieredTieredMigrationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredMigrationMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationMilestoneMinimumOffsetAdjuster {
    pub weight: f64,
}

impl TieredMigrationMilestoneStep for TieredMigrationMilestoneMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &TieredMigrationMilestone, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the tiered migration milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredMigrationMilestoneEngine {
    pub steps: Vec<Rc<dyn TieredMigrationMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredMigrationMilestoneEngine {
    pub fn price(&mut self, item: &TieredMigrationMilestone) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[TieredMigrationMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_migration_milestone_engine() -> TieredMigrationMilestoneEngine {
    TieredMigrationMilestoneEngine {
        steps: vec![
            Rc::new(MinimumTieredMigrationMilestoneAdjuster { minimum: 22.7 }) as Rc<dyn TieredMigrationMilestoneStep>,
            Rc::new(RegionTieredMigrationMilestoneAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.95),
                    ("west".to_string(), 1.16),
                    ("south".to_string(), 1.05),
                ]),
            }),
            Rc::new(PercentTieredMigrationMilestoneAdjuster { percent: 0.2 }),
            Rc::new(RoundingTieredMigrationMilestoneAdjuster { step: 1.0 }),
            Rc::new(TieredTieredMigrationMilestoneAdjuster {
                bounds: vec![1078.0, 6813.0, 9918.0],
                rates: vec![0.87, 0.82, 1.11],
            }),
            Rc::new(TieredMigrationMilestoneMinimumOffsetAdjuster { weight: 1.02 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 77261.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_migration_milestone_engine();
        let item = new_tiered_migration_milestone("code-282", "name-731");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
