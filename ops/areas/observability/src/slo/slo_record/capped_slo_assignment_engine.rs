use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped slo assignment calculation.
pub trait CappedSloAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedSloAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloAssignmentLimitOffsetAdjuster {
    pub weight: f64,
}

impl CappedSloAssignmentStep for CappedSloAssignmentLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &CappedSloAssignment, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedSloAssignmentAdjuster {
    pub percent: f64,
}

impl CappedSloAssignmentStep for PercentCappedSloAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedSloAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedSloAssignmentAdjuster {
    pub step: f64,
}

impl CappedSloAssignmentStep for RoundingCappedSloAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedSloAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedSloAssignmentAdjuster {
    pub cap: f64,
}

impl CappedSloAssignmentStep for CapCappedSloAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedSloAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedSloAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedSloAssignmentStep for FlaggedCappedSloAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedSloAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleCappedSloAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedSloAssignmentStep for TitleCappedSloAssignmentAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &CappedSloAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the capped slo assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedSloAssignmentEngine {
    pub steps: Vec<Rc<dyn CappedSloAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedSloAssignmentEngine {
    pub fn compute(&mut self, item: &CappedSloAssignment) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedSloAssignmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedSloAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_slo_assignment_engine(flags: &flags::Registry) -> CappedSloAssignmentEngine {
    CappedSloAssignmentEngine {
        steps: vec![
            Rc::new(CappedSloAssignmentLimitOffsetAdjuster { weight: 0.62 }) as Rc<dyn CappedSloAssignmentStep>,
            Rc::new(PercentCappedSloAssignmentAdjuster { percent: 4.3 }),
            Rc::new(RoundingCappedSloAssignmentAdjuster { step: 1.0 }),
            Rc::new(CapCappedSloAssignmentAdjuster { cap: 45967.0 }),
            Rc::new(FlaggedCappedSloAssignmentAdjuster {
                flags: flags.clone(),
                flag: "capped-slo-assignment-manual".to_string(),
                percent: -1.0,
            }),
            Rc::new(TitleCappedSloAssignmentAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.22),
                    ("north".to_string(), 0.97),
                    ("east".to_string(), 1.21),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 7914.4,
    }
}
