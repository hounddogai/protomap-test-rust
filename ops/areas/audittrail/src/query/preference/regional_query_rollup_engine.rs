use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional query rollup calculation.
pub trait RegionalQueryRollupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalQueryRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryRollupDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl RegionalQueryRollupStep for RegionalQueryRollupDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalQueryRollup, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalQueryRollupAdjuster {
    pub cap: f64,
}

impl RegionalQueryRollupStep for CapRegionalQueryRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalQueryRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalQueryRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalQueryRollupStep for FlaggedRegionalQueryRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalQueryRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalQueryRollupAdjuster {
    pub minimum: f64,
}

impl RegionalQueryRollupStep for MinimumRegionalQueryRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalQueryRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional query rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalQueryRollupEngine {
    pub steps: Vec<Rc<dyn RegionalQueryRollupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalQueryRollupEngine {
    pub fn compute(&mut self, item: &RegionalQueryRollup) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[RegionalQueryRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_query_rollup_engine(flags: &flags::Registry) -> RegionalQueryRollupEngine {
    RegionalQueryRollupEngine {
        steps: vec![
            Rc::new(RegionalQueryRollupDurationMinutesOffsetAdjuster { weight: 1.47 })
                as Rc<dyn RegionalQueryRollupStep>,
            Rc::new(CapRegionalQueryRollupAdjuster { cap: 14522.0 }),
            Rc::new(FlaggedRegionalQueryRollupAdjuster {
                flags: flags.clone(),
                flag: "regional-query-rollup-fallback".to_string(),
                percent: -6.1,
            }),
            Rc::new(MinimumRegionalQueryRollupAdjuster { minimum: 9.7 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 92643.0,
    }
}
