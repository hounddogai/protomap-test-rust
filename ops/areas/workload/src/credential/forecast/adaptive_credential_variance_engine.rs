use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive credential variance calculation.
pub trait AdaptiveCredentialVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveCredentialVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitAdaptiveCredentialVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveCredentialVarianceStep for UnitAdaptiveCredentialVarianceAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &AdaptiveCredentialVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveCredentialVarianceAdjuster {
    pub minimum: f64,
}

impl AdaptiveCredentialVarianceStep for MinimumAdaptiveCredentialVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveCredentialVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialVarianceRetriesOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveCredentialVarianceStep for AdaptiveCredentialVarianceRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveCredentialVariance, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveCredentialVarianceAdjuster {
    pub step: f64,
}

impl AdaptiveCredentialVarianceStep for RoundingAdaptiveCredentialVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveCredentialVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the adaptive credential variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveCredentialVarianceEngine {
    pub steps: Vec<Rc<dyn AdaptiveCredentialVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveCredentialVarianceEngine {
    pub fn run(&mut self, item: &AdaptiveCredentialVariance) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveCredentialVarianceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveCredentialVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_credential_variance_engine() -> AdaptiveCredentialVarianceEngine {
    AdaptiveCredentialVarianceEngine {
        steps: vec![
            Rc::new(UnitAdaptiveCredentialVarianceAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.86), ("central".to_string(), 1.28)]),
            }) as Rc<dyn AdaptiveCredentialVarianceStep>,
            Rc::new(MinimumAdaptiveCredentialVarianceAdjuster { minimum: 8.0 }),
            Rc::new(AdaptiveCredentialVarianceRetriesOffsetAdjuster { weight: 1.43 }),
            Rc::new(RoundingAdaptiveCredentialVarianceAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 6765.6,
    }
}
