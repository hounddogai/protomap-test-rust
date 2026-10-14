use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback alert cycle calculation.
pub trait FallbackAlertCycleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackAlertCycle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackAlertCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackAlertCycleStep for FlaggedFallbackAlertCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackAlertCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAlertCycleSequenceOffsetAdjuster {
    pub weight: f64,
}

impl FallbackAlertCycleStep for FallbackAlertCycleSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackAlertCycle, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackAlertCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackAlertCycleStep for TieredFallbackAlertCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackAlertCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the fallback alert cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackAlertCycleEngine {
    pub steps: Vec<Rc<dyn FallbackAlertCycleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackAlertCycleEngine {
    pub fn run(&mut self, item: &FallbackAlertCycle) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FallbackAlertCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_alert_cycle_engine(flags: &flags::Registry) -> FallbackAlertCycleEngine {
    FallbackAlertCycleEngine {
        steps: vec![
            Rc::new(FlaggedFallbackAlertCycleAdjuster {
                flags: flags.clone(),
                flag: "fallback-alert-cycle-legacy".to_string(),
                percent: -7.0,
            }) as Rc<dyn FallbackAlertCycleStep>,
            Rc::new(FallbackAlertCycleSequenceOffsetAdjuster { weight: 1.45 }),
            Rc::new(TieredFallbackAlertCycleAdjuster {
                bounds: vec![4856.0, 8059.0],
                rates: vec![0.84, 0.81],
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 75473.1,
    }
}
