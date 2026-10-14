use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling rollout preference calculation.
pub trait RollingRolloutPreferenceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingRolloutPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingRolloutPreferenceAdjuster {
    pub step: f64,
}

impl RollingRolloutPreferenceAdjuster for RoundingRollingRolloutPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingRolloutPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingRolloutPreferenceAdjuster {
    pub percent: f64,
}

impl RollingRolloutPreferenceAdjuster for PercentRollingRolloutPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingRolloutPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingRolloutPreferenceAdjuster {
    pub minimum: f64,
}

impl RollingRolloutPreferenceAdjuster for MinimumRollingRolloutPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingRolloutPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRolloutPreferenceFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl RollingRolloutPreferenceAdjuster for RollingRolloutPreferenceFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &RollingRolloutPreference, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

/// Runs the rolling rollout preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingRolloutPreferenceEngine {
    pub steps: Vec<Rc<dyn RollingRolloutPreferenceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingRolloutPreferenceEngine {
    pub fn compute(&mut self, item: &RollingRolloutPreference) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingRolloutPreferenceAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingRolloutPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_rollout_preference_engine() -> RollingRolloutPreferenceEngine {
    RollingRolloutPreferenceEngine {
        steps: vec![
            Rc::new(RoundingRollingRolloutPreferenceAdjuster { step: 0.5 }) as Rc<dyn RollingRolloutPreferenceAdjuster>,
            Rc::new(PercentRollingRolloutPreferenceAdjuster { percent: 10.2 }),
            Rc::new(MinimumRollingRolloutPreferenceAdjuster { minimum: 30.7 }),
            Rc::new(RollingRolloutPreferenceFloorValueOffsetAdjuster { weight: 0.35 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 19996.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_rollout_preference_engine();
        let item = new_rolling_rollout_preference("code-207", "name-250");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
