use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rolling timeline budget calculation.
pub trait RollingTimelineBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingTimelineBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingTimelineBudgetAdjuster {
    pub cap: f64,
}

impl RollingTimelineBudgetStep for CapRollingTimelineBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingTimelineBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingTimelineBudgetAdjuster {
    pub percent: f64,
}

impl RollingTimelineBudgetStep for PercentRollingTimelineBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingTimelineBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRollingTimelineBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingTimelineBudgetStep for NameRollingTimelineBudgetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RollingTimelineBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingTimelineBudgetAdjuster {
    pub minimum: f64,
}

impl RollingTimelineBudgetStep for MinimumRollingTimelineBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingTimelineBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingTimelineBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingTimelineBudgetStep for FlaggedRollingTimelineBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingTimelineBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTimelineBudgetMinimumOffsetAdjuster {
    pub weight: f64,
}

impl RollingTimelineBudgetStep for RollingTimelineBudgetMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &RollingTimelineBudget, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the rolling timeline budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingTimelineBudgetEngine {
    pub steps: Vec<Rc<dyn RollingTimelineBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingTimelineBudgetEngine {
    pub fn evaluate(&mut self, item: &RollingTimelineBudget) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingTimelineBudgetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingTimelineBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_timeline_budget_engine(flags: &flags::Registry) -> RollingTimelineBudgetEngine {
    RollingTimelineBudgetEngine {
        steps: vec![
            Rc::new(CapRollingTimelineBudgetAdjuster { cap: 17907.0 }) as Rc<dyn RollingTimelineBudgetStep>,
            Rc::new(PercentRollingTimelineBudgetAdjuster { percent: 2.2 }),
            Rc::new(NameRollingTimelineBudgetAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.95)]),
            }),
            Rc::new(MinimumRollingTimelineBudgetAdjuster { minimum: 36.8 }),
            Rc::new(FlaggedRollingTimelineBudgetAdjuster {
                flags: flags.clone(),
                flag: "rolling-timeline-budget-lenient".to_string(),
                percent: 7.2,
            }),
            Rc::new(RollingTimelineBudgetMinimumOffsetAdjuster { weight: 1.28 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 8961.4,
    }
}
