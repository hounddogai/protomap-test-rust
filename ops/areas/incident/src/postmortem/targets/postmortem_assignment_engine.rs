use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the postmortem assignment calculation.
pub trait PostmortemAssignmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPostmortemAssignmentAdjuster {
    pub minimum: f64,
}

impl PostmortemAssignmentFactor for MinimumPostmortemAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PostmortemAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPostmortemAssignmentAdjuster {
    pub percent: f64,
}

impl PostmortemAssignmentFactor for PercentPostmortemAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PostmortemAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemAssignmentFactor for FlaggedPostmortemAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAssignmentBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl PostmortemAssignmentFactor for PostmortemAssignmentBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &PostmortemAssignment, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

/// Runs the postmortem assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemAssignmentEngine {
    pub steps: Vec<Rc<dyn PostmortemAssignmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemAssignmentEngine {
    pub fn run(&mut self, item: &PostmortemAssignment) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PostmortemAssignmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_postmortem_assignment_engine(flags: &flags::Registry) -> PostmortemAssignmentEngine {
    PostmortemAssignmentEngine {
        steps: vec![
            Rc::new(MinimumPostmortemAssignmentAdjuster { minimum: 45.6 }) as Rc<dyn PostmortemAssignmentFactor>,
            Rc::new(PercentPostmortemAssignmentAdjuster { percent: 6.6 }),
            Rc::new(FlaggedPostmortemAssignmentAdjuster {
                flags: flags.clone(),
                flag: "postmortem-assignment-global".to_string(),
                percent: -2.1,
            }),
            Rc::new(PostmortemAssignmentBatchSizeOffsetAdjuster { weight: 1.78 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 19332.1,
    }
}
