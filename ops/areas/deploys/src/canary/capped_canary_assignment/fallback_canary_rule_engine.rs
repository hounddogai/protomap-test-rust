use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback canary rule calculation.
pub trait FallbackCanaryRuleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackCanaryRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackCanaryRuleAdjuster {
    pub step: f64,
}

impl FallbackCanaryRuleFactor for RoundingFallbackCanaryRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackCanaryRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackCanaryRuleAdjuster {
    pub percent: f64,
}

impl FallbackCanaryRuleFactor for PercentFallbackCanaryRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackCanaryRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentFallbackCanaryRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackCanaryRuleFactor for SegmentFallbackCanaryRuleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &FallbackCanaryRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackCanaryRuleAdjuster {
    pub cap: f64,
}

impl FallbackCanaryRuleFactor for CapFallbackCanaryRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackCanaryRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackCanaryRuleCeilingOffsetAdjuster {
    pub weight: f64,
}

impl FallbackCanaryRuleFactor for FallbackCanaryRuleCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackCanaryRule, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackCanaryRuleAdjuster {
    pub minimum: f64,
}

impl FallbackCanaryRuleFactor for MinimumFallbackCanaryRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackCanaryRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the fallback canary rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackCanaryRuleEngine {
    pub steps: Vec<Rc<dyn FallbackCanaryRuleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackCanaryRuleEngine {
    pub fn evaluate(&mut self, item: &FallbackCanaryRule) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackCanaryRuleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackCanaryRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_canary_rule_engine() -> FallbackCanaryRuleEngine {
    FallbackCanaryRuleEngine {
        steps: vec![
            Rc::new(RoundingFallbackCanaryRuleAdjuster { step: 1.0 }) as Rc<dyn FallbackCanaryRuleFactor>,
            Rc::new(PercentFallbackCanaryRuleAdjuster { percent: -24.0 }),
            Rc::new(SegmentFallbackCanaryRuleAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.92), ("south".to_string(), 1.02)]),
            }),
            Rc::new(CapFallbackCanaryRuleAdjuster { cap: 33857.0 }),
            Rc::new(FallbackCanaryRuleCeilingOffsetAdjuster { weight: 0.85 }),
            Rc::new(MinimumFallbackCanaryRuleAdjuster { minimum: 24.9 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 23467.0,
    }
}
