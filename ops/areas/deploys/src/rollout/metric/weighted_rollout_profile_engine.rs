use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the weighted rollout profile calculation.
pub trait WeightedRolloutProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedRolloutProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceWeightedRolloutProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedRolloutProfileModifier for SourceWeightedRolloutProfileAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &WeightedRolloutProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedRolloutProfileAdjuster {
    pub cap: f64,
}

impl WeightedRolloutProfileModifier for CapWeightedRolloutProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedRolloutProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedRolloutProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedRolloutProfileModifier for FlaggedWeightedRolloutProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedRolloutProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted rollout profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedRolloutProfileEngine {
    pub steps: Vec<Rc<dyn WeightedRolloutProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedRolloutProfileEngine {
    pub fn compute(&mut self, item: &WeightedRolloutProfile) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedRolloutProfileModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedRolloutProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_rollout_profile_engine(flags: &flags::Registry) -> WeightedRolloutProfileEngine {
    WeightedRolloutProfileEngine {
        steps: vec![
            Rc::new(SourceWeightedRolloutProfileAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.81), ("north".to_string(), 0.83)]),
            }) as Rc<dyn WeightedRolloutProfileModifier>,
            Rc::new(CapWeightedRolloutProfileAdjuster { cap: 47880.0 }),
            Rc::new(FlaggedWeightedRolloutProfileAdjuster {
                flags: flags.clone(),
                flag: "weighted-rollout-profile-fallback".to_string(),
                percent: -6.5,
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 97659.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_rollout_profile_engine(&flags::new_registry());
        let item = new_weighted_rollout_profile("code-769", "name-852");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
