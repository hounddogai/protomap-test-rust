use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary segment group calculation.
pub trait PrimarySegmentGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimarySegmentGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimarySegmentGroupAdjuster {
    pub step: f64,
}

impl PrimarySegmentGroupStep for RoundingPrimarySegmentGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimarySegmentGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimarySegmentGroupAdjuster {
    pub cap: f64,
}

impl PrimarySegmentGroupStep for CapPrimarySegmentGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimarySegmentGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimarySegmentGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimarySegmentGroupStep for FlaggedPrimarySegmentGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimarySegmentGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary segment group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimarySegmentGroupEngine {
    pub steps: Vec<Rc<dyn PrimarySegmentGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimarySegmentGroupEngine {
    pub fn evaluate(&mut self, item: &PrimarySegmentGroup) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimarySegmentGroupStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimarySegmentGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_segment_group_engine(flags: &flags::Registry) -> PrimarySegmentGroupEngine {
    PrimarySegmentGroupEngine {
        steps: vec![
            Rc::new(RoundingPrimarySegmentGroupAdjuster { step: 0.1 }) as Rc<dyn PrimarySegmentGroupStep>,
            Rc::new(CapPrimarySegmentGroupAdjuster { cap: 23550.0 }),
            Rc::new(FlaggedPrimarySegmentGroupAdjuster {
                flags: flags.clone(),
                flag: "primary-segment-group-regional".to_string(),
                percent: 7.6,
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 50556.7,
    }
}
