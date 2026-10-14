use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default release rule calculation.
pub trait DefaultReleaseRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultReleaseRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultReleaseRuleAdjuster {
    pub minimum: f64,
}

impl DefaultReleaseRuleStep for MinimumDefaultReleaseRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultReleaseRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultReleaseRuleAdjuster {
    pub step: f64,
}

impl DefaultReleaseRuleStep for RoundingDefaultReleaseRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultReleaseRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseRuleExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl DefaultReleaseRuleStep for DefaultReleaseRuleExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultReleaseRule, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeDefaultReleaseRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultReleaseRuleStep for ScopeDefaultReleaseRuleAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &DefaultReleaseRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default release rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultReleaseRuleEngine {
    pub steps: Vec<Rc<dyn DefaultReleaseRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultReleaseRuleEngine {
    pub fn evaluate(&mut self, item: &DefaultReleaseRule) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultReleaseRuleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultReleaseRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_release_rule_engine() -> DefaultReleaseRuleEngine {
    DefaultReleaseRuleEngine {
        steps: vec![
            Rc::new(MinimumDefaultReleaseRuleAdjuster { minimum: 28.7 }) as Rc<dyn DefaultReleaseRuleStep>,
            Rc::new(RoundingDefaultReleaseRuleAdjuster { step: 0.5 }),
            Rc::new(DefaultReleaseRuleExpiresAtOffsetAdjuster { weight: 1.13 }),
            Rc::new(ScopeDefaultReleaseRuleAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.97),
                    ("north".to_string(), 0.81),
                    ("retail".to_string(), 0.98),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 51770.0,
    }
}
