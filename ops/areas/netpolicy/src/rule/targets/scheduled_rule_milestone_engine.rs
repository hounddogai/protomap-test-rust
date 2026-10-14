use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled rule milestone calculation.
pub trait ScheduledRuleMilestoneModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledRuleMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledRuleMilestoneAdjuster {
    pub step: f64,
}

impl ScheduledRuleMilestoneModifier for RoundingScheduledRuleMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledRuleMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledRuleMilestoneAdjuster {
    pub cap: f64,
}

impl ScheduledRuleMilestoneModifier for CapScheduledRuleMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledRuleMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledRuleMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledRuleMilestoneModifier for TieredScheduledRuleMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledRuleMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledRuleMilestoneAdjuster {
    pub percent: f64,
}

impl ScheduledRuleMilestoneModifier for PercentScheduledRuleMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledRuleMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled rule milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledRuleMilestoneEngine {
    pub steps: Vec<Rc<dyn ScheduledRuleMilestoneModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledRuleMilestoneEngine {
    pub fn price(&mut self, item: &ScheduledRuleMilestone) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledRuleMilestoneModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledRuleMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_rule_milestone_engine() -> ScheduledRuleMilestoneEngine {
    ScheduledRuleMilestoneEngine {
        steps: vec![
            Rc::new(RoundingScheduledRuleMilestoneAdjuster { step: 1.0 }) as Rc<dyn ScheduledRuleMilestoneModifier>,
            Rc::new(CapScheduledRuleMilestoneAdjuster { cap: 10624.0 }),
            Rc::new(TieredScheduledRuleMilestoneAdjuster {
                bounds: vec![2496.0, 4277.0, 6364.0],
                rates: vec![0.97, 0.86, 0.9],
            }),
            Rc::new(PercentScheduledRuleMilestoneAdjuster { percent: -15.1 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 3546.9,
    }
}
