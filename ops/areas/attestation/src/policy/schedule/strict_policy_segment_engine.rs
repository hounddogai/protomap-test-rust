use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the strict policy segment calculation.
pub trait StrictPolicySegmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictPolicySegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictPolicySegmentAdjuster {
    pub minimum: f64,
}

impl StrictPolicySegmentFactor for MinimumStrictPolicySegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictPolicySegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictPolicySegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictPolicySegmentFactor for FlaggedStrictPolicySegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictPolicySegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictPolicySegmentAdjuster {
    pub percent: f64,
}

impl StrictPolicySegmentFactor for PercentStrictPolicySegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictPolicySegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict policy segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictPolicySegmentEngine {
    pub steps: Vec<Rc<dyn StrictPolicySegmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictPolicySegmentEngine {
    pub fn evaluate(&mut self, item: &StrictPolicySegment) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictPolicySegmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictPolicySegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_policy_segment_engine(flags: &flags::Registry) -> StrictPolicySegmentEngine {
    StrictPolicySegmentEngine {
        steps: vec![
            Rc::new(MinimumStrictPolicySegmentAdjuster { minimum: 46.0 }) as Rc<dyn StrictPolicySegmentFactor>,
            Rc::new(FlaggedStrictPolicySegmentAdjuster {
                flags: flags.clone(),
                flag: "strict-policy-segment-scheduled".to_string(),
                percent: -4.3,
            }),
            Rc::new(PercentStrictPolicySegmentAdjuster { percent: 10.3 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 31213.4,
    }
}
