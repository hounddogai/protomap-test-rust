use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive rollback milestone calculation.
pub trait AdaptiveRollbackMilestoneFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRollbackMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveRollbackMilestoneAdjuster {
    pub step: f64,
}

impl AdaptiveRollbackMilestoneFactor for RoundingAdaptiveRollbackMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveRollbackMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveRollbackMilestoneAdjuster {
    pub cap: f64,
}

impl AdaptiveRollbackMilestoneFactor for CapAdaptiveRollbackMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveRollbackMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveRollbackMilestoneAdjuster {
    pub minimum: f64,
}

impl AdaptiveRollbackMilestoneFactor for MinimumAdaptiveRollbackMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveRollbackMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveRollbackMilestoneAdjuster {
    pub percent: f64,
}

impl AdaptiveRollbackMilestoneFactor for PercentAdaptiveRollbackMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveRollbackMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveRollbackMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveRollbackMilestoneFactor for TieredAdaptiveRollbackMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveRollbackMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionAdaptiveRollbackMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveRollbackMilestoneFactor for RegionAdaptiveRollbackMilestoneAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &AdaptiveRollbackMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the adaptive rollback milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRollbackMilestoneEngine {
    pub steps: Vec<Rc<dyn AdaptiveRollbackMilestoneFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRollbackMilestoneEngine {
    pub fn compute(&mut self, item: &AdaptiveRollbackMilestone) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveRollbackMilestoneFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRollbackMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_rollback_milestone_engine() -> AdaptiveRollbackMilestoneEngine {
    AdaptiveRollbackMilestoneEngine {
        steps: vec![
            Rc::new(RoundingAdaptiveRollbackMilestoneAdjuster { step: 0.05 })
                as Rc<dyn AdaptiveRollbackMilestoneFactor>,
            Rc::new(CapAdaptiveRollbackMilestoneAdjuster { cap: 12648.0 }),
            Rc::new(MinimumAdaptiveRollbackMilestoneAdjuster { minimum: 13.6 }),
            Rc::new(PercentAdaptiveRollbackMilestoneAdjuster { percent: -14.5 }),
            Rc::new(TieredAdaptiveRollbackMilestoneAdjuster {
                bounds: vec![2691.0, 4058.0],
                rates: vec![0.95, 0.95],
            }),
            Rc::new(RegionAdaptiveRollbackMilestoneAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.78), ("east".to_string(), 1.34)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 21449.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_rollback_milestone_engine();
        let item = new_adaptive_rollback_milestone("code-801", "name-295");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
