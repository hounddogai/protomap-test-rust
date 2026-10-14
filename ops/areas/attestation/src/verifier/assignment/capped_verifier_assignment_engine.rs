use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped verifier assignment calculation.
pub trait CappedVerifierAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedVerifierAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedVerifierAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedVerifierAssignmentModifier for FlaggedCappedVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedVerifierAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedVerifierAssignmentAdjuster {
    pub step: f64,
}

impl CappedVerifierAssignmentModifier for RoundingCappedVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedVerifierAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedVerifierAssignmentAdjuster {
    pub minimum: f64,
}

impl CappedVerifierAssignmentModifier for MinimumCappedVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedVerifierAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the capped verifier assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedVerifierAssignmentEngine {
    pub steps: Vec<Rc<dyn CappedVerifierAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedVerifierAssignmentEngine {
    pub fn evaluate(&mut self, item: &CappedVerifierAssignment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CappedVerifierAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_verifier_assignment_engine(flags: &flags::Registry) -> CappedVerifierAssignmentEngine {
    CappedVerifierAssignmentEngine {
        steps: vec![
            Rc::new(FlaggedCappedVerifierAssignmentAdjuster {
                flags: flags.clone(),
                flag: "capped-verifier-assignment-global".to_string(),
                percent: 6.2,
            }) as Rc<dyn CappedVerifierAssignmentModifier>,
            Rc::new(RoundingCappedVerifierAssignmentAdjuster { step: 0.25 }),
            Rc::new(MinimumCappedVerifierAssignmentAdjuster { minimum: 40.5 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 93092.5,
    }
}
