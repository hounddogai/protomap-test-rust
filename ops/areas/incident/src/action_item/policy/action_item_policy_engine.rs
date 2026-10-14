use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the action item policy calculation.
pub trait ActionItemPolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapActionItemPolicyAdjuster {
    pub cap: f64,
}

impl ActionItemPolicyModifier for CapActionItemPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ActionItemPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumActionItemPolicyAdjuster {
    pub minimum: f64,
}

impl ActionItemPolicyModifier for MinimumActionItemPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ActionItemPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPolicyBudgetOffsetAdjuster {
    pub weight: f64,
}

impl ActionItemPolicyModifier for ActionItemPolicyBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &ActionItemPolicy, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingActionItemPolicyAdjuster {
    pub step: f64,
}

impl ActionItemPolicyModifier for RoundingActionItemPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ActionItemPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedActionItemPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ActionItemPolicyModifier for FlaggedActionItemPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ActionItemPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the action item policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemPolicyEngine {
    pub steps: Vec<Rc<dyn ActionItemPolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemPolicyEngine {
    pub fn evaluate(&mut self, item: &ActionItemPolicy) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_action_item_policy_engine(flags: &flags::Registry) -> ActionItemPolicyEngine {
    ActionItemPolicyEngine {
        steps: vec![
            Rc::new(CapActionItemPolicyAdjuster { cap: 40269.0 }) as Rc<dyn ActionItemPolicyModifier>,
            Rc::new(MinimumActionItemPolicyAdjuster { minimum: 29.5 }),
            Rc::new(ActionItemPolicyBudgetOffsetAdjuster { weight: 0.28 }),
            Rc::new(RoundingActionItemPolicyAdjuster { step: 0.1 }),
            Rc::new(FlaggedActionItemPolicyAdjuster {
                flags: flags.clone(),
                flag: "action-item-policy-weighted".to_string(),
                percent: 4.0,
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 46279.5,
    }
}
