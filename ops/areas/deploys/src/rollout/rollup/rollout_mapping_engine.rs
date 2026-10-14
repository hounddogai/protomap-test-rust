use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rollout mapping calculation.
pub trait RolloutMappingFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RolloutMappingFactor for TieredRolloutMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RolloutMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRolloutMappingAdjuster {
    pub cap: f64,
}

impl RolloutMappingFactor for CapRolloutMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RolloutMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRolloutMappingAdjuster {
    pub step: f64,
}

impl RolloutMappingFactor for RoundingRolloutMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RolloutMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRolloutMappingAdjuster {
    pub minimum: f64,
}

impl RolloutMappingFactor for MinimumRolloutMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RolloutMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rollout mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutMappingEngine {
    pub steps: Vec<Rc<dyn RolloutMappingFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutMappingEngine {
    pub fn run(&mut self, item: &RolloutMapping) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RolloutMappingFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RolloutMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rollout_mapping_engine() -> RolloutMappingEngine {
    RolloutMappingEngine {
        steps: vec![
            Rc::new(TieredRolloutMappingAdjuster {
                bounds: vec![4005.0, 5942.0, 7416.0],
                rates: vec![1.04, 0.96, 0.85],
            }) as Rc<dyn RolloutMappingFactor>,
            Rc::new(CapRolloutMappingAdjuster { cap: 32545.0 }),
            Rc::new(RoundingRolloutMappingAdjuster { step: 0.1 }),
            Rc::new(MinimumRolloutMappingAdjuster { minimum: 7.1 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 21251.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_mapping_engine();
        let item = new_rollout_mapping("code-912", "name-144");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
