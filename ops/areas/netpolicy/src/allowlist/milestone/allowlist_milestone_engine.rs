use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the allowlist milestone calculation.
pub trait AllowlistMilestoneModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAllowlistMilestoneAdjuster {
    pub cap: f64,
}

impl AllowlistMilestoneModifier for CapAllowlistMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AllowlistMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAllowlistMilestoneAdjuster {
    pub step: f64,
}

impl AllowlistMilestoneModifier for RoundingAllowlistMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AllowlistMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAllowlistMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AllowlistMilestoneModifier for FlaggedAllowlistMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AllowlistMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the allowlist milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistMilestoneEngine {
    pub steps: Vec<Rc<dyn AllowlistMilestoneModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistMilestoneEngine {
    pub fn price(&mut self, item: &AllowlistMilestone) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AllowlistMilestoneModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_allowlist_milestone_engine(flags: &flags::Registry) -> AllowlistMilestoneEngine {
    AllowlistMilestoneEngine {
        steps: vec![
            Rc::new(CapAllowlistMilestoneAdjuster { cap: 11054.0 }) as Rc<dyn AllowlistMilestoneModifier>,
            Rc::new(RoundingAllowlistMilestoneAdjuster { step: 0.5 }),
            Rc::new(FlaggedAllowlistMilestoneAdjuster {
                flags: flags.clone(),
                flag: "allowlist-milestone-default".to_string(),
                percent: 7.6,
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 3360.1,
    }
}
