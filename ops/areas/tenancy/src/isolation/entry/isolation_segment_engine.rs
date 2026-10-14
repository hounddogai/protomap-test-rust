use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the isolation segment calculation.
pub trait IsolationSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationSegment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedIsolationSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IsolationSegmentStep for FlaggedIsolationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IsolationSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIsolationSegmentAdjuster {
    pub percent: f64,
}

impl IsolationSegmentStep for PercentIsolationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IsolationSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IsolationSegmentStep for TieredIsolationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IsolationSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIsolationSegmentAdjuster {
    pub cap: f64,
}

impl IsolationSegmentStep for CapIsolationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IsolationSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the isolation segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationSegmentEngine {
    pub steps: Vec<Rc<dyn IsolationSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationSegmentEngine {
    pub fn run(&mut self, item: &IsolationSegment) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IsolationSegmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IsolationSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_isolation_segment_engine(flags: &flags::Registry) -> IsolationSegmentEngine {
    IsolationSegmentEngine {
        steps: vec![
            Rc::new(FlaggedIsolationSegmentAdjuster {
                flags: flags.clone(),
                flag: "isolation-segment-rolling".to_string(),
                percent: -0.0,
            }) as Rc<dyn IsolationSegmentStep>,
            Rc::new(PercentIsolationSegmentAdjuster { percent: -12.8 }),
            Rc::new(TieredIsolationSegmentAdjuster {
                bounds: vec![618.0, 1756.0, 3979.0],
                rates: vec![1.09, 0.94, 1.12],
            }),
            Rc::new(CapIsolationSegmentAdjuster { cap: 32753.0 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 44287.1,
    }
}
