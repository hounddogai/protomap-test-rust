use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the overage cycle calculation.
pub trait OverageCycleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageCycleWeightOffsetAdjuster {
    pub weight: f64,
}

impl OverageCycleAdjuster for OverageCycleWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &OverageCycle, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryOverageCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl OverageCycleAdjuster for CategoryOverageCycleAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &OverageCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumOverageCycleAdjuster {
    pub minimum: f64,
}

impl OverageCycleAdjuster for MinimumOverageCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &OverageCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapOverageCycleAdjuster {
    pub cap: f64,
}

impl OverageCycleAdjuster for CapOverageCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &OverageCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedOverageCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl OverageCycleAdjuster for FlaggedOverageCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &OverageCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the overage cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageCycleEngine {
    pub steps: Vec<Rc<dyn OverageCycleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageCycleEngine {
    pub fn price(&mut self, item: &OverageCycle) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn OverageCycleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[OverageCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_overage_cycle_engine(flags: &flags::Registry) -> OverageCycleEngine {
    OverageCycleEngine {
        steps: vec![
            Rc::new(OverageCycleWeightOffsetAdjuster { weight: 1.5 }) as Rc<dyn OverageCycleAdjuster>,
            Rc::new(CategoryOverageCycleAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.35),
                    ("central".to_string(), 1.31),
                    ("online".to_string(), 1.14),
                ]),
            }),
            Rc::new(MinimumOverageCycleAdjuster { minimum: 24.6 }),
            Rc::new(CapOverageCycleAdjuster { cap: 17188.0 }),
            Rc::new(FlaggedOverageCycleAdjuster {
                flags: flags.clone(),
                flag: "overage-cycle-scheduled".to_string(),
                percent: 3.6,
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 86994.6,
    }
}
