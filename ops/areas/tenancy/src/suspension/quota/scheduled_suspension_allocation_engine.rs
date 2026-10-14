use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled suspension allocation calculation.
pub trait ScheduledSuspensionAllocationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledSuspensionAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledSuspensionAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledSuspensionAllocationAdjuster for FlaggedScheduledSuspensionAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledSuspensionAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledSuspensionAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledSuspensionAllocationAdjuster for TieredScheduledSuspensionAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledSuspensionAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledSuspensionAllocationAdjuster {
    pub cap: f64,
}

impl ScheduledSuspensionAllocationAdjuster for CapScheduledSuspensionAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledSuspensionAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledSuspensionAllocationAdjuster {
    pub percent: f64,
}

impl ScheduledSuspensionAllocationAdjuster for PercentScheduledSuspensionAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledSuspensionAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledSuspensionAllocationAdjuster {
    pub step: f64,
}

impl ScheduledSuspensionAllocationAdjuster for RoundingScheduledSuspensionAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledSuspensionAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledSuspensionAllocationAdjuster {
    pub minimum: f64,
}

impl ScheduledSuspensionAllocationAdjuster for MinimumScheduledSuspensionAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledSuspensionAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the scheduled suspension allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledSuspensionAllocationEngine {
    pub steps: Vec<Rc<dyn ScheduledSuspensionAllocationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledSuspensionAllocationEngine {
    pub fn compute(&mut self, item: &ScheduledSuspensionAllocation) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledSuspensionAllocationAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledSuspensionAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_suspension_allocation_engine(flags: &flags::Registry) -> ScheduledSuspensionAllocationEngine {
    ScheduledSuspensionAllocationEngine {
        steps: vec![
            Rc::new(FlaggedScheduledSuspensionAllocationAdjuster {
                flags: flags.clone(),
                flag: "scheduled-suspension-allocation-weighted".to_string(),
                percent: -5.3,
            }) as Rc<dyn ScheduledSuspensionAllocationAdjuster>,
            Rc::new(TieredScheduledSuspensionAllocationAdjuster {
                bounds: vec![3835.0, 6383.0, 7051.0],
                rates: vec![1.2, 0.86, 0.93],
            }),
            Rc::new(CapScheduledSuspensionAllocationAdjuster { cap: 21012.0 }),
            Rc::new(PercentScheduledSuspensionAllocationAdjuster { percent: -5.1 }),
            Rc::new(RoundingScheduledSuspensionAllocationAdjuster { step: 5.0 }),
            Rc::new(MinimumScheduledSuspensionAllocationAdjuster { minimum: 13.0 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 97836.1,
    }
}
