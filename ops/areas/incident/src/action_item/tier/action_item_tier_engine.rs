use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the action item tier calculation.
pub trait ActionItemTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTierRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl ActionItemTierFactor for ActionItemTierRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &ActionItemTier, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentActionItemTierAdjuster {
    pub percent: f64,
}

impl ActionItemTierFactor for PercentActionItemTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ActionItemTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelActionItemTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ActionItemTierFactor for LabelActionItemTierAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ActionItemTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumActionItemTierAdjuster {
    pub minimum: f64,
}

impl ActionItemTierFactor for MinimumActionItemTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ActionItemTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the action item tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemTierEngine {
    pub steps: Vec<Rc<dyn ActionItemTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemTierEngine {
    pub fn run(&mut self, item: &ActionItemTier) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ActionItemTierFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_action_item_tier_engine() -> ActionItemTierEngine {
    ActionItemTierEngine {
        steps: vec![
            Rc::new(ActionItemTierRotationDaysOffsetAdjuster { weight: 1.23 }) as Rc<dyn ActionItemTierFactor>,
            Rc::new(PercentActionItemTierAdjuster { percent: 3.4 }),
            Rc::new(LabelActionItemTierAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.75), ("north".to_string(), 0.9)]),
            }),
            Rc::new(MinimumActionItemTierAdjuster { minimum: 19.2 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 67322.0,
    }
}
