use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the canary assignment calculation.
pub trait CanaryAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCanaryAssignmentAdjuster {
    pub percent: f64,
}

impl CanaryAssignmentAdjuster for PercentCanaryAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CanaryAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCanaryAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CanaryAssignmentAdjuster for FlaggedCanaryAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CanaryAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAssignmentBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl CanaryAssignmentAdjuster for CanaryAssignmentBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &CanaryAssignment, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CanaryAssignmentAdjuster for TieredCanaryAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CanaryAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitCanaryAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CanaryAssignmentAdjuster for UnitCanaryAssignmentAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &CanaryAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCanaryAssignmentAdjuster {
    pub cap: f64,
}

impl CanaryAssignmentAdjuster for CapCanaryAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CanaryAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the canary assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryAssignmentEngine {
    pub steps: Vec<Rc<dyn CanaryAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryAssignmentEngine {
    pub fn evaluate(&mut self, item: &CanaryAssignment) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[CanaryAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_canary_assignment_engine(flags: &flags::Registry) -> CanaryAssignmentEngine {
    CanaryAssignmentEngine {
        steps: vec![
            Rc::new(PercentCanaryAssignmentAdjuster { percent: 8.4 }) as Rc<dyn CanaryAssignmentAdjuster>,
            Rc::new(FlaggedCanaryAssignmentAdjuster {
                flags: flags.clone(),
                flag: "canary-assignment-default".to_string(),
                percent: -1.2,
            }),
            Rc::new(CanaryAssignmentBatchSizeOffsetAdjuster { weight: 0.63 }),
            Rc::new(TieredCanaryAssignmentAdjuster {
                bounds: vec![1977.0, 3605.0, 4021.0, 6506.0],
                rates: vec![1.13, 0.95, 0.9, 1.0],
            }),
            Rc::new(UnitCanaryAssignmentAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.36),
                    ("east".to_string(), 0.84),
                    ("west".to_string(), 1.15),
                ]),
            }),
            Rc::new(CapCanaryAssignmentAdjuster { cap: 3722.0 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 77982.7,
    }
}
