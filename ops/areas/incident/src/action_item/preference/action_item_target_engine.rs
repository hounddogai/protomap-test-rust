use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the action item target calculation.
pub trait ActionItemTargetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTargetAmountOffsetAdjuster {
    pub weight: f64,
}

impl ActionItemTargetStep for ActionItemTargetAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &ActionItemTarget, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentActionItemTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ActionItemTargetStep for SegmentActionItemTargetAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &ActionItemTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumActionItemTargetAdjuster {
    pub minimum: f64,
}

impl ActionItemTargetStep for MinimumActionItemTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ActionItemTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingActionItemTargetAdjuster {
    pub step: f64,
}

impl ActionItemTargetStep for RoundingActionItemTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ActionItemTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedActionItemTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ActionItemTargetStep for FlaggedActionItemTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ActionItemTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ActionItemTargetStep for TieredActionItemTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ActionItemTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the action item target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemTargetEngine {
    pub steps: Vec<Rc<dyn ActionItemTargetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemTargetEngine {
    pub fn price(&mut self, item: &ActionItemTarget) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ActionItemTargetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_action_item_target_engine(flags: &flags::Registry) -> ActionItemTargetEngine {
    ActionItemTargetEngine {
        steps: vec![
            Rc::new(ActionItemTargetAmountOffsetAdjuster { weight: 1.26 }) as Rc<dyn ActionItemTargetStep>,
            Rc::new(SegmentActionItemTargetAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.0)]),
            }),
            Rc::new(MinimumActionItemTargetAdjuster { minimum: 22.1 }),
            Rc::new(RoundingActionItemTargetAdjuster { step: 0.05 }),
            Rc::new(FlaggedActionItemTargetAdjuster {
                flags: flags.clone(),
                flag: "action-item-target-lenient".to_string(),
                percent: -0.9,
            }),
            Rc::new(TieredActionItemTargetAdjuster {
                bounds: vec![1555.0, 1563.0, 6947.0, 8307.0],
                rates: vec![1.08, 1.18, 1.04, 1.2],
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 53438.8,
    }
}
