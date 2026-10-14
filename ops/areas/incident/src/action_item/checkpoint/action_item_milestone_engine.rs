use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the action item milestone calculation.
pub trait ActionItemMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ActionItemMilestoneStep for TieredActionItemMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ActionItemMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedActionItemMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ActionItemMilestoneStep for FlaggedActionItemMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ActionItemMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapActionItemMilestoneAdjuster {
    pub cap: f64,
}

impl ActionItemMilestoneStep for CapActionItemMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ActionItemMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumActionItemMilestoneAdjuster {
    pub minimum: f64,
}

impl ActionItemMilestoneStep for MinimumActionItemMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ActionItemMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the action item milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemMilestoneEngine {
    pub steps: Vec<Rc<dyn ActionItemMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemMilestoneEngine {
    pub fn price(&mut self, item: &ActionItemMilestone) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ActionItemMilestoneStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_action_item_milestone_engine(flags: &flags::Registry) -> ActionItemMilestoneEngine {
    ActionItemMilestoneEngine {
        steps: vec![
            Rc::new(TieredActionItemMilestoneAdjuster {
                bounds: vec![4786.0, 7791.0, 9343.0],
                rates: vec![0.83, 0.87, 0.94],
            }) as Rc<dyn ActionItemMilestoneStep>,
            Rc::new(FlaggedActionItemMilestoneAdjuster {
                flags: flags.clone(),
                flag: "action-item-milestone-capped".to_string(),
                percent: 2.0,
            }),
            Rc::new(CapActionItemMilestoneAdjuster { cap: 12477.0 }),
            Rc::new(MinimumActionItemMilestoneAdjuster { minimum: 41.7 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 64194.1,
    }
}
