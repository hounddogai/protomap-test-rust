use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the query assignment calculation.
pub trait QueryAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQueryAssignmentAdjuster {
    pub step: f64,
}

impl QueryAssignmentAdjuster for RoundingQueryAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QueryAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryAssignmentAdjuster for FlaggedQueryAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQueryAssignmentAdjuster {
    pub minimum: f64,
}

impl QueryAssignmentAdjuster for MinimumQueryAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QueryAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the query assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryAssignmentEngine {
    pub steps: Vec<Rc<dyn QueryAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryAssignmentEngine {
    pub fn run(&mut self, item: &QueryAssignment) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[QueryAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_query_assignment_engine(flags: &flags::Registry) -> QueryAssignmentEngine {
    QueryAssignmentEngine {
        steps: vec![
            Rc::new(RoundingQueryAssignmentAdjuster { step: 0.1 }) as Rc<dyn QueryAssignmentAdjuster>,
            Rc::new(FlaggedQueryAssignmentAdjuster {
                flags: flags.clone(),
                flag: "query-assignment-global".to_string(),
                percent: -6.4,
            }),
            Rc::new(MinimumQueryAssignmentAdjuster { minimum: 12.8 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 34512.6,
    }
}
