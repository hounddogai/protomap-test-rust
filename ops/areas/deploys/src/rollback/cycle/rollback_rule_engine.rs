use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rollback rule calculation.
pub trait RollbackRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackRuleAdjuster {
    pub percent: f64,
}

impl RollbackRuleStep for PercentRollbackRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollbackRuleAdjuster {
    pub cap: f64,
}

impl RollbackRuleStep for CapRollbackRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollbackRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRollbackRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollbackRuleStep for LabelRollbackRuleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RollbackRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackRuleAdjuster {
    pub minimum: f64,
}

impl RollbackRuleStep for MinimumRollbackRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rollback rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackRuleEngine {
    pub steps: Vec<Rc<dyn RollbackRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackRuleEngine {
    pub fn compute(&mut self, item: &RollbackRule) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[RollbackRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rollback_rule_engine() -> RollbackRuleEngine {
    RollbackRuleEngine {
        steps: vec![
            Rc::new(PercentRollbackRuleAdjuster { percent: 14.0 }) as Rc<dyn RollbackRuleStep>,
            Rc::new(CapRollbackRuleAdjuster { cap: 1480.0 }),
            Rc::new(LabelRollbackRuleAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.85)]),
            }),
            Rc::new(MinimumRollbackRuleAdjuster { minimum: 4.7 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 61633.7,
    }
}
