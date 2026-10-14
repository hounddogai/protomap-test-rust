use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global release assignment calculation.
pub trait GlobalReleaseAssignmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalReleaseAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalReleaseAssignmentAdjuster {
    pub step: f64,
}

impl GlobalReleaseAssignmentFactor for RoundingGlobalReleaseAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalReleaseAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAssignmentCeilingOffsetAdjuster {
    pub weight: f64,
}

impl GlobalReleaseAssignmentFactor for GlobalReleaseAssignmentCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalReleaseAssignment, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalReleaseAssignmentAdjuster {
    pub minimum: f64,
}

impl GlobalReleaseAssignmentFactor for MinimumGlobalReleaseAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalReleaseAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalReleaseAssignmentAdjuster {
    pub cap: f64,
}

impl GlobalReleaseAssignmentFactor for CapGlobalReleaseAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalReleaseAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalReleaseAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalReleaseAssignmentFactor for FlaggedGlobalReleaseAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalReleaseAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global release assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalReleaseAssignmentEngine {
    pub steps: Vec<Rc<dyn GlobalReleaseAssignmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalReleaseAssignmentEngine {
    pub fn price(&mut self, item: &GlobalReleaseAssignment) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalReleaseAssignmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalReleaseAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_release_assignment_engine(flags: &flags::Registry) -> GlobalReleaseAssignmentEngine {
    GlobalReleaseAssignmentEngine {
        steps: vec![
            Rc::new(RoundingGlobalReleaseAssignmentAdjuster { step: 0.5 }) as Rc<dyn GlobalReleaseAssignmentFactor>,
            Rc::new(GlobalReleaseAssignmentCeilingOffsetAdjuster { weight: 0.52 }),
            Rc::new(MinimumGlobalReleaseAssignmentAdjuster { minimum: 47.2 }),
            Rc::new(CapGlobalReleaseAssignmentAdjuster { cap: 21458.0 }),
            Rc::new(FlaggedGlobalReleaseAssignmentAdjuster {
                flags: flags.clone(),
                flag: "global-release-assignment-manual".to_string(),
                percent: -0.2,
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 37842.5,
    }
}
