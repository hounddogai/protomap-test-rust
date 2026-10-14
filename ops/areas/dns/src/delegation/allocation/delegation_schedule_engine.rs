use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the delegation schedule calculation.
pub trait DelegationScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDelegationScheduleAdjuster {
    pub step: f64,
}

impl DelegationScheduleStep for RoundingDelegationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DelegationSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDelegationScheduleAdjuster {
    pub cap: f64,
}

impl DelegationScheduleStep for CapDelegationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DelegationSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationScheduleStep for TieredDelegationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationScheduleStep for FlaggedDelegationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationScheduleAdjuster {
    pub minimum: f64,
}

impl DelegationScheduleStep for MinimumDelegationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDelegationScheduleAdjuster {
    pub percent: f64,
}

impl DelegationScheduleStep for PercentDelegationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DelegationSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the delegation schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationScheduleEngine {
    pub steps: Vec<Rc<dyn DelegationScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationScheduleEngine {
    pub fn run(&mut self, item: &DelegationSchedule) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DelegationScheduleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DelegationSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_delegation_schedule_engine(flags: &flags::Registry) -> DelegationScheduleEngine {
    DelegationScheduleEngine {
        steps: vec![
            Rc::new(RoundingDelegationScheduleAdjuster { step: 0.25 }) as Rc<dyn DelegationScheduleStep>,
            Rc::new(CapDelegationScheduleAdjuster { cap: 8366.0 }),
            Rc::new(TieredDelegationScheduleAdjuster {
                bounds: vec![326.0, 985.0, 4203.0, 6296.0],
                rates: vec![1.03, 0.91, 0.81, 1.11],
            }),
            Rc::new(FlaggedDelegationScheduleAdjuster {
                flags: flags.clone(),
                flag: "delegation-schedule-pending".to_string(),
                percent: 3.9,
            }),
            Rc::new(MinimumDelegationScheduleAdjuster { minimum: 0.2 }),
            Rc::new(PercentDelegationScheduleAdjuster { percent: 14.5 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 58070.7,
    }
}
