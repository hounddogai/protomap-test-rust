use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global plan window calculation.
pub trait GlobalPlanWindowStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalPlanWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalPlanWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalPlanWindowStep for TieredGlobalPlanWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalPlanWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalPlanWindowAdjuster {
    pub cap: f64,
}

impl GlobalPlanWindowStep for CapGlobalPlanWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalPlanWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalPlanWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalPlanWindowStep for FlaggedGlobalPlanWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalPlanWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanWindowLimitOffsetAdjuster {
    pub weight: f64,
}

impl GlobalPlanWindowStep for GlobalPlanWindowLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalPlanWindow, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

/// Runs the global plan window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalPlanWindowEngine {
    pub steps: Vec<Rc<dyn GlobalPlanWindowStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalPlanWindowEngine {
    pub fn price(&mut self, item: &GlobalPlanWindow) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[GlobalPlanWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_plan_window_engine(flags: &flags::Registry) -> GlobalPlanWindowEngine {
    GlobalPlanWindowEngine {
        steps: vec![
            Rc::new(TieredGlobalPlanWindowAdjuster {
                bounds: vec![1861.0, 3846.0, 6204.0, 7420.0],
                rates: vec![0.95, 1.11, 0.8, 0.99],
            }) as Rc<dyn GlobalPlanWindowStep>,
            Rc::new(CapGlobalPlanWindowAdjuster { cap: 13779.0 }),
            Rc::new(FlaggedGlobalPlanWindowAdjuster {
                flags: flags.clone(),
                flag: "global-plan-window-regional".to_string(),
                percent: -5.6,
            }),
            Rc::new(GlobalPlanWindowLimitOffsetAdjuster { weight: 0.51 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 35956.8,
    }
}
