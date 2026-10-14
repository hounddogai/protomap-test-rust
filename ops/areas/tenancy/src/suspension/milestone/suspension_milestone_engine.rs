use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the suspension milestone calculation.
pub trait SuspensionMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSuspensionMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SuspensionMilestoneStep for LabelSuspensionMilestoneAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &SuspensionMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSuspensionMilestoneAdjuster {
    pub cap: f64,
}

impl SuspensionMilestoneStep for CapSuspensionMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SuspensionMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSuspensionMilestoneAdjuster {
    pub minimum: f64,
}

impl SuspensionMilestoneStep for MinimumSuspensionMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SuspensionMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the suspension milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionMilestoneEngine {
    pub steps: Vec<Rc<dyn SuspensionMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionMilestoneEngine {
    pub fn price(&mut self, item: &SuspensionMilestone) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SuspensionMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_suspension_milestone_engine() -> SuspensionMilestoneEngine {
    SuspensionMilestoneEngine {
        steps: vec![
            Rc::new(LabelSuspensionMilestoneAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.0),
                    ("west".to_string(), 1.09),
                    ("online".to_string(), 1.33),
                ]),
            }) as Rc<dyn SuspensionMilestoneStep>,
            Rc::new(CapSuspensionMilestoneAdjuster { cap: 13468.0 }),
            Rc::new(MinimumSuspensionMilestoneAdjuster { minimum: 45.9 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 68400.0,
    }
}
