use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending verifier assignment calculation.
pub trait PendingVerifierAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingVerifierAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingVerifierAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingVerifierAssignmentModifier for FlaggedPendingVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingVerifierAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingVerifierAssignmentAdjuster {
    pub percent: f64,
}

impl PendingVerifierAssignmentModifier for PercentPendingVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingVerifierAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingVerifierAssignmentAdjuster {
    pub minimum: f64,
}

impl PendingVerifierAssignmentModifier for MinimumPendingVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingVerifierAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingVerifierAssignmentAdjuster {
    pub step: f64,
}

impl PendingVerifierAssignmentModifier for RoundingPendingVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingVerifierAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingVerifierAssignmentAdjuster {
    pub cap: f64,
}

impl PendingVerifierAssignmentModifier for CapPendingVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingVerifierAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the pending verifier assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingVerifierAssignmentEngine {
    pub steps: Vec<Rc<dyn PendingVerifierAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingVerifierAssignmentEngine {
    pub fn run(&mut self, item: &PendingVerifierAssignment) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingVerifierAssignmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingVerifierAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_verifier_assignment_engine(flags: &flags::Registry) -> PendingVerifierAssignmentEngine {
    PendingVerifierAssignmentEngine {
        steps: vec![
            Rc::new(FlaggedPendingVerifierAssignmentAdjuster {
                flags: flags.clone(),
                flag: "pending-verifier-assignment-pending".to_string(),
                percent: 0.0,
            }) as Rc<dyn PendingVerifierAssignmentModifier>,
            Rc::new(PercentPendingVerifierAssignmentAdjuster { percent: -6.8 }),
            Rc::new(MinimumPendingVerifierAssignmentAdjuster { minimum: 33.1 }),
            Rc::new(RoundingPendingVerifierAssignmentAdjuster { step: 0.1 }),
            Rc::new(CapPendingVerifierAssignmentAdjuster { cap: 24392.0 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 67171.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_verifier_assignment_engine(&flags::new_registry());
        let item = new_pending_verifier_assignment("code-424", "title-841");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
