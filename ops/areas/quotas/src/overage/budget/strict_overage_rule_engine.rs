use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict overage rule calculation.
pub trait StrictOverageRuleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictOverageRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageRuleSequenceOffsetAdjuster {
    pub weight: f64,
}

impl StrictOverageRuleFactor for StrictOverageRuleSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &StrictOverageRule, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictOverageRuleAdjuster {
    pub cap: f64,
}

impl StrictOverageRuleFactor for CapStrictOverageRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictOverageRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelStrictOverageRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictOverageRuleFactor for ChannelStrictOverageRuleAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &StrictOverageRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the strict overage rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictOverageRuleEngine {
    pub steps: Vec<Rc<dyn StrictOverageRuleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictOverageRuleEngine {
    pub fn evaluate(&mut self, item: &StrictOverageRule) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[StrictOverageRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_overage_rule_engine() -> StrictOverageRuleEngine {
    StrictOverageRuleEngine {
        steps: vec![
            Rc::new(StrictOverageRuleSequenceOffsetAdjuster { weight: 0.91 }) as Rc<dyn StrictOverageRuleFactor>,
            Rc::new(CapStrictOverageRuleAdjuster { cap: 43240.0 }),
            Rc::new(ChannelStrictOverageRuleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.23)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 26107.6,
    }
}
