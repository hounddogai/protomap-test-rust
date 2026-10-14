use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the adaptive rule assignment calculation.
pub trait AdaptiveRuleAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRuleAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveRuleAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveRuleAssignmentStep for FlaggedAdaptiveRuleAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveRuleAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleAssignmentDiscountOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveRuleAssignmentStep for AdaptiveRuleAssignmentDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveRuleAssignment, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveRuleAssignmentAdjuster {
    pub minimum: f64,
}

impl AdaptiveRuleAssignmentStep for MinimumAdaptiveRuleAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveRuleAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the adaptive rule assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRuleAssignmentEngine {
    pub steps: Vec<Rc<dyn AdaptiveRuleAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRuleAssignmentEngine {
    pub fn price(&mut self, item: &AdaptiveRuleAssignment) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveRuleAssignmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRuleAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_rule_assignment_engine(flags: &flags::Registry) -> AdaptiveRuleAssignmentEngine {
    AdaptiveRuleAssignmentEngine {
        steps: vec![
            Rc::new(FlaggedAdaptiveRuleAssignmentAdjuster {
                flags: flags.clone(),
                flag: "adaptive-rule-assignment-manual".to_string(),
                percent: -9.1,
            }) as Rc<dyn AdaptiveRuleAssignmentStep>,
            Rc::new(AdaptiveRuleAssignmentDiscountOffsetAdjuster { weight: 0.54 }),
            Rc::new(MinimumAdaptiveRuleAssignmentAdjuster { minimum: 26.8 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 57398.9,
    }
}
