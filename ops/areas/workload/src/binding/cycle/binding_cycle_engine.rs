use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the binding cycle calculation.
pub trait BindingCycleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingCycleQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl BindingCycleStep for BindingCycleQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &BindingCycle, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBindingCycleAdjuster {
    pub percent: f64,
}

impl BindingCycleStep for PercentBindingCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BindingCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBindingCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BindingCycleStep for FlaggedBindingCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BindingCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBindingCycleAdjuster {
    pub cap: f64,
}

impl BindingCycleStep for CapBindingCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BindingCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BindingCycleStep for TieredBindingCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BindingCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the binding cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingCycleEngine {
    pub steps: Vec<Rc<dyn BindingCycleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingCycleEngine {
    pub fn evaluate(&mut self, item: &BindingCycle) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BindingCycleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BindingCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_binding_cycle_engine(flags: &flags::Registry) -> BindingCycleEngine {
    BindingCycleEngine {
        steps: vec![
            Rc::new(BindingCycleQuotaUnitsOffsetAdjuster { weight: 0.12 }) as Rc<dyn BindingCycleStep>,
            Rc::new(PercentBindingCycleAdjuster { percent: -22.3 }),
            Rc::new(FlaggedBindingCycleAdjuster {
                flags: flags.clone(),
                flag: "binding-cycle-capped".to_string(),
                percent: 3.8,
            }),
            Rc::new(CapBindingCycleAdjuster { cap: 33467.0 }),
            Rc::new(TieredBindingCycleAdjuster {
                bounds: vec![5176.0, 6009.0, 7078.0, 9626.0],
                rates: vec![0.94, 1.13, 1.12, 0.81],
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 28761.6,
    }
}
