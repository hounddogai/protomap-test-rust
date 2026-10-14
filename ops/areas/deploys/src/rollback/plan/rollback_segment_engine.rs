use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollback segment calculation.
pub trait RollbackSegmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackSegmentAdjuster {
    pub minimum: f64,
}

impl RollbackSegmentModifier for MinimumRollbackSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollbackSegmentAdjuster {
    pub step: f64,
}

impl RollbackSegmentModifier for RoundingRollbackSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollbackSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollbackSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollbackSegmentModifier for FlaggedRollbackSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollbackSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rollback segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackSegmentEngine {
    pub steps: Vec<Rc<dyn RollbackSegmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackSegmentEngine {
    pub fn compute(&mut self, item: &RollbackSegment) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[RollbackSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rollback_segment_engine(flags: &flags::Registry) -> RollbackSegmentEngine {
    RollbackSegmentEngine {
        steps: vec![
            Rc::new(MinimumRollbackSegmentAdjuster { minimum: 17.6 }) as Rc<dyn RollbackSegmentModifier>,
            Rc::new(RoundingRollbackSegmentAdjuster { step: 0.5 }),
            Rc::new(FlaggedRollbackSegmentAdjuster {
                flags: flags.clone(),
                flag: "rollback-segment-default".to_string(),
                percent: 8.4,
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 15014.2,
    }
}
