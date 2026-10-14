use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the regional timeline cycle calculation.
pub trait RegionalTimelineCycleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalTimelineCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalTimelineCycleAdjuster {
    pub cap: f64,
}

impl RegionalTimelineCycleAdjuster for CapRegionalTimelineCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalTimelineCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalTimelineCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalTimelineCycleAdjuster for FlaggedRegionalTimelineCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalTimelineCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalTimelineCycleAdjuster {
    pub minimum: f64,
}

impl RegionalTimelineCycleAdjuster for MinimumRegionalTimelineCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalTimelineCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalTimelineCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalTimelineCycleAdjuster for TieredRegionalTimelineCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalTimelineCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineCycleRevisionOffsetAdjuster {
    pub weight: f64,
}

impl RegionalTimelineCycleAdjuster for RegionalTimelineCycleRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalTimelineCycle, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalTimelineCycleAdjuster {
    pub percent: f64,
}

impl RegionalTimelineCycleAdjuster for PercentRegionalTimelineCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalTimelineCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional timeline cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalTimelineCycleEngine {
    pub steps: Vec<Rc<dyn RegionalTimelineCycleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalTimelineCycleEngine {
    pub fn price(&mut self, item: &RegionalTimelineCycle) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalTimelineCycleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalTimelineCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_timeline_cycle_engine(flags: &flags::Registry) -> RegionalTimelineCycleEngine {
    RegionalTimelineCycleEngine {
        steps: vec![
            Rc::new(CapRegionalTimelineCycleAdjuster { cap: 26743.0 }) as Rc<dyn RegionalTimelineCycleAdjuster>,
            Rc::new(FlaggedRegionalTimelineCycleAdjuster {
                flags: flags.clone(),
                flag: "regional-timeline-cycle-regional".to_string(),
                percent: 7.6,
            }),
            Rc::new(MinimumRegionalTimelineCycleAdjuster { minimum: 19.1 }),
            Rc::new(TieredRegionalTimelineCycleAdjuster {
                bounds: vec![7080.0, 7937.0],
                rates: vec![0.82, 1.02],
            }),
            Rc::new(RegionalTimelineCycleRevisionOffsetAdjuster { weight: 0.32 }),
            Rc::new(PercentRegionalTimelineCycleAdjuster { percent: -11.4 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 58079.4,
    }
}
