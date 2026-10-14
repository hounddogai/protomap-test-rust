use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rule item calculation.
pub trait RuleItemAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleItem, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRuleItemAdjuster {
    pub percent: f64,
}

impl RuleItemAdjuster for PercentRuleItemAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RuleItem, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRuleItemAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RuleItemAdjuster for FlaggedRuleItemAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RuleItem, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleItemCostOffsetAdjuster {
    pub weight: f64,
}

impl RuleItemAdjuster for RuleItemCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &RuleItem, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

/// Runs the rule item steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleItemEngine {
    pub steps: Vec<Rc<dyn RuleItemAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleItemEngine {
    pub fn evaluate(&mut self, item: &RuleItem) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RuleItemAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RuleItem]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rule_item_engine(flags: &flags::Registry) -> RuleItemEngine {
    RuleItemEngine {
        steps: vec![
            Rc::new(PercentRuleItemAdjuster { percent: 4.0 }) as Rc<dyn RuleItemAdjuster>,
            Rc::new(FlaggedRuleItemAdjuster {
                flags: flags.clone(),
                flag: "rule-item-batched".to_string(),
                percent: -3.3,
            }),
            Rc::new(RuleItemCostOffsetAdjuster { weight: 1.66 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 57470.0,
    }
}
