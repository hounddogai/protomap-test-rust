use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive federation cycle calculation.
pub trait AdaptiveFederationCycleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveFederationCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveFederationCycleAdjuster {
    pub step: f64,
}

impl AdaptiveFederationCycleStep for RoundingAdaptiveFederationCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveFederationCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelAdaptiveFederationCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveFederationCycleStep for ChannelAdaptiveFederationCycleAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &AdaptiveFederationCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFederationCycleLimitOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveFederationCycleStep for AdaptiveFederationCycleLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveFederationCycle, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

/// Runs the adaptive federation cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveFederationCycleEngine {
    pub steps: Vec<Rc<dyn AdaptiveFederationCycleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveFederationCycleEngine {
    pub fn run(&mut self, item: &AdaptiveFederationCycle) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveFederationCycleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveFederationCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_federation_cycle_engine() -> AdaptiveFederationCycleEngine {
    AdaptiveFederationCycleEngine {
        steps: vec![
            Rc::new(RoundingAdaptiveFederationCycleAdjuster { step: 0.5 }) as Rc<dyn AdaptiveFederationCycleStep>,
            Rc::new(ChannelAdaptiveFederationCycleAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.89), ("online".to_string(), 0.94)]),
            }),
            Rc::new(AdaptiveFederationCycleLimitOffsetAdjuster { weight: 0.71 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 31711.7,
    }
}
