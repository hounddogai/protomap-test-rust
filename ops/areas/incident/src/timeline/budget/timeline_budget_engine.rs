use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the timeline budget calculation.
pub trait TimelineBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTimelineBudgetAdjuster {
    pub cap: f64,
}

impl TimelineBudgetAdjuster for CapTimelineBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TimelineBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TimelineBudgetAdjuster for TieredTimelineBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TimelineBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBudgetWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl TimelineBudgetAdjuster for TimelineBudgetWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &TimelineBudget, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTimelineBudgetAdjuster {
    pub minimum: f64,
}

impl TimelineBudgetAdjuster for MinimumTimelineBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TimelineBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTimelineBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TimelineBudgetAdjuster for FlaggedTimelineBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TimelineBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the timeline budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineBudgetEngine {
    pub steps: Vec<Rc<dyn TimelineBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineBudgetEngine {
    pub fn compute(&mut self, item: &TimelineBudget) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[TimelineBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_timeline_budget_engine(flags: &flags::Registry) -> TimelineBudgetEngine {
    TimelineBudgetEngine {
        steps: vec![
            Rc::new(CapTimelineBudgetAdjuster { cap: 45359.0 }) as Rc<dyn TimelineBudgetAdjuster>,
            Rc::new(TieredTimelineBudgetAdjuster {
                bounds: vec![4115.0, 4497.0, 7397.0],
                rates: vec![0.83, 1.07, 0.81],
            }),
            Rc::new(TimelineBudgetWindowDaysOffsetAdjuster { weight: 0.77 }),
            Rc::new(MinimumTimelineBudgetAdjuster { minimum: 8.1 }),
            Rc::new(FlaggedTimelineBudgetAdjuster {
                flags: flags.clone(),
                flag: "timeline-budget-strict".to_string(),
                percent: 4.2,
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 29814.3,
    }
}
