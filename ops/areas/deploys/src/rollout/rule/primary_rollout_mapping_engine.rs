use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary rollout mapping calculation.
pub trait PrimaryRolloutMappingAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryRolloutMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryRolloutMappingAdjuster {
    pub step: f64,
}

impl PrimaryRolloutMappingAdjuster for RoundingPrimaryRolloutMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryRolloutMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryRolloutMappingAdjuster {
    pub minimum: f64,
}

impl PrimaryRolloutMappingAdjuster for MinimumPrimaryRolloutMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryRolloutMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutMappingRatioOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryRolloutMappingAdjuster for PrimaryRolloutMappingRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryRolloutMapping, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

/// Runs the primary rollout mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryRolloutMappingEngine {
    pub steps: Vec<Rc<dyn PrimaryRolloutMappingAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryRolloutMappingEngine {
    pub fn evaluate(&mut self, item: &PrimaryRolloutMapping) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryRolloutMappingAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryRolloutMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_rollout_mapping_engine() -> PrimaryRolloutMappingEngine {
    PrimaryRolloutMappingEngine {
        steps: vec![
            Rc::new(RoundingPrimaryRolloutMappingAdjuster { step: 0.1 }) as Rc<dyn PrimaryRolloutMappingAdjuster>,
            Rc::new(MinimumPrimaryRolloutMappingAdjuster { minimum: 32.5 }),
            Rc::new(PrimaryRolloutMappingRatioOffsetAdjuster { weight: 1.99 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 88905.1,
    }
}
