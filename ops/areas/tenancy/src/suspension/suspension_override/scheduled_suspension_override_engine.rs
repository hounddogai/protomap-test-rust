use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the scheduled suspension override calculation.
pub trait ScheduledSuspensionOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledSuspensionOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleScheduledSuspensionOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledSuspensionOverrideModifier for TitleScheduledSuspensionOverrideAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ScheduledSuspensionOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionOverrideCostOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledSuspensionOverrideModifier for ScheduledSuspensionOverrideCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledSuspensionOverride, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledSuspensionOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledSuspensionOverrideModifier for FlaggedScheduledSuspensionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledSuspensionOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledSuspensionOverrideAdjuster {
    pub cap: f64,
}

impl ScheduledSuspensionOverrideModifier for CapScheduledSuspensionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledSuspensionOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the scheduled suspension override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledSuspensionOverrideEngine {
    pub steps: Vec<Rc<dyn ScheduledSuspensionOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledSuspensionOverrideEngine {
    pub fn compute(&mut self, item: &ScheduledSuspensionOverride) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledSuspensionOverrideModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledSuspensionOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_suspension_override_engine(flags: &flags::Registry) -> ScheduledSuspensionOverrideEngine {
    ScheduledSuspensionOverrideEngine {
        steps: vec![
            Rc::new(TitleScheduledSuspensionOverrideAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.2)]),
            }) as Rc<dyn ScheduledSuspensionOverrideModifier>,
            Rc::new(ScheduledSuspensionOverrideCostOffsetAdjuster { weight: 1.16 }),
            Rc::new(FlaggedScheduledSuspensionOverrideAdjuster {
                flags: flags.clone(),
                flag: "scheduled-suspension-override-global".to_string(),
                percent: -8.4,
            }),
            Rc::new(CapScheduledSuspensionOverrideAdjuster { cap: 25327.0 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 6865.9,
    }
}
