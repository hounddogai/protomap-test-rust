use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy rule preference calculation.
pub trait LegacyRulePreferenceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyRulePreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyRulePreferenceAdjuster {
    pub step: f64,
}

impl LegacyRulePreferenceFactor for RoundingLegacyRulePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyRulePreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyRulePreferenceAdjuster {
    pub percent: f64,
}

impl LegacyRulePreferenceFactor for PercentLegacyRulePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyRulePreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRulePreferencePortOffsetAdjuster {
    pub weight: f64,
}

impl LegacyRulePreferenceFactor for LegacyRulePreferencePortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyRulePreference, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

/// Runs the legacy rule preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyRulePreferenceEngine {
    pub steps: Vec<Rc<dyn LegacyRulePreferenceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyRulePreferenceEngine {
    pub fn evaluate(&mut self, item: &LegacyRulePreference) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyRulePreferenceFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyRulePreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_rule_preference_engine() -> LegacyRulePreferenceEngine {
    LegacyRulePreferenceEngine {
        steps: vec![
            Rc::new(RoundingLegacyRulePreferenceAdjuster { step: 0.1 }) as Rc<dyn LegacyRulePreferenceFactor>,
            Rc::new(PercentLegacyRulePreferenceAdjuster { percent: 21.1 }),
            Rc::new(LegacyRulePreferencePortOffsetAdjuster { weight: 1.46 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 41357.7,
    }
}
