use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the verifier milestone calculation.
pub trait VerifierMilestoneModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentVerifierMilestoneAdjuster {
    pub percent: f64,
}

impl VerifierMilestoneModifier for PercentVerifierMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &VerifierMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMilestoneEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl VerifierMilestoneModifier for VerifierMilestoneEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &VerifierMilestone, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierMilestoneAdjuster {
    pub step: f64,
}

impl VerifierMilestoneModifier for RoundingVerifierMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the verifier milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierMilestoneEngine {
    pub steps: Vec<Rc<dyn VerifierMilestoneModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierMilestoneEngine {
    pub fn run(&mut self, item: &VerifierMilestone) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn VerifierMilestoneModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[VerifierMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_verifier_milestone_engine() -> VerifierMilestoneEngine {
    VerifierMilestoneEngine {
        steps: vec![
            Rc::new(PercentVerifierMilestoneAdjuster { percent: -19.4 }) as Rc<dyn VerifierMilestoneModifier>,
            Rc::new(VerifierMilestoneEndsAtOffsetAdjuster { weight: 0.41 }),
            Rc::new(RoundingVerifierMilestoneAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 28881.0,
    }
}
