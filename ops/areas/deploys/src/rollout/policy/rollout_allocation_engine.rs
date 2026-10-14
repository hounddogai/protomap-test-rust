use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rollout allocation calculation.
pub trait RolloutAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRolloutAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutAllocationModifier for CategoryRolloutAllocationAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RolloutAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutAllocationModifier for FlaggedRolloutAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRolloutAllocationAdjuster {
    pub percent: f64,
}

impl RolloutAllocationModifier for PercentRolloutAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RolloutAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAllocationCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl RolloutAllocationModifier for RolloutAllocationCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &RolloutAllocation, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRolloutAllocationAdjuster {
    pub cap: f64,
}

impl RolloutAllocationModifier for CapRolloutAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RolloutAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRolloutAllocationAdjuster {
    pub minimum: f64,
}

impl RolloutAllocationModifier for MinimumRolloutAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RolloutAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rollout allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutAllocationEngine {
    pub steps: Vec<Rc<dyn RolloutAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutAllocationEngine {
    pub fn price(&mut self, item: &RolloutAllocation) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RolloutAllocationModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RolloutAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rollout_allocation_engine(flags: &flags::Registry) -> RolloutAllocationEngine {
    RolloutAllocationEngine {
        steps: vec![
            Rc::new(CategoryRolloutAllocationAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.27)]),
            }) as Rc<dyn RolloutAllocationModifier>,
            Rc::new(FlaggedRolloutAllocationAdjuster {
                flags: flags.clone(),
                flag: "rollout-allocation-pending".to_string(),
                percent: 2.6,
            }),
            Rc::new(PercentRolloutAllocationAdjuster { percent: 3.5 }),
            Rc::new(RolloutAllocationCreatedAtOffsetAdjuster { weight: 1.53 }),
            Rc::new(CapRolloutAllocationAdjuster { cap: 39587.0 }),
            Rc::new(MinimumRolloutAllocationAdjuster { minimum: 23.8 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 17652.1,
    }
}
