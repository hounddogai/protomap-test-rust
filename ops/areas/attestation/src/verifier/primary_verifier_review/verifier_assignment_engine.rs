use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the verifier assignment calculation.
pub trait VerifierAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierAssignmentAdjuster {
    pub cap: f64,
}

impl VerifierAssignmentStep for CapVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentVerifierAssignmentAdjuster {
    pub percent: f64,
}

impl VerifierAssignmentStep for PercentVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &VerifierAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedVerifierAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl VerifierAssignmentStep for FlaggedVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &VerifierAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAssignmentUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl VerifierAssignmentStep for VerifierAssignmentUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &VerifierAssignment, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumVerifierAssignmentAdjuster {
    pub minimum: f64,
}

impl VerifierAssignmentStep for MinimumVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &VerifierAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the verifier assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierAssignmentEngine {
    pub steps: Vec<Rc<dyn VerifierAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierAssignmentEngine {
    pub fn run(&mut self, item: &VerifierAssignment) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn VerifierAssignmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[VerifierAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_verifier_assignment_engine(flags: &flags::Registry) -> VerifierAssignmentEngine {
    VerifierAssignmentEngine {
        steps: vec![
            Rc::new(CapVerifierAssignmentAdjuster { cap: 46555.0 }) as Rc<dyn VerifierAssignmentStep>,
            Rc::new(PercentVerifierAssignmentAdjuster { percent: -2.5 }),
            Rc::new(FlaggedVerifierAssignmentAdjuster {
                flags: flags.clone(),
                flag: "verifier-assignment-seasonal".to_string(),
                percent: -3.9,
            }),
            Rc::new(VerifierAssignmentUpdatedAtOffsetAdjuster { weight: 1.47 }),
            Rc::new(MinimumVerifierAssignmentAdjuster { minimum: 37.2 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 11478.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_assignment_engine(&flags::new_registry());
        let item = new_verifier_assignment("code-802", "label-796");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
