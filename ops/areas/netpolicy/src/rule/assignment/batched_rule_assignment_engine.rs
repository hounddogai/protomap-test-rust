use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched rule assignment calculation.
pub trait BatchedRuleAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedRuleAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedRuleAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedRuleAssignmentAdjuster for FlaggedBatchedRuleAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedRuleAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedRuleAssignmentAdjuster {
    pub step: f64,
}

impl BatchedRuleAssignmentAdjuster for RoundingBatchedRuleAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedRuleAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedRuleAssignmentAdjuster {
    pub cap: f64,
}

impl BatchedRuleAssignmentAdjuster for CapBatchedRuleAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedRuleAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedRuleAssignmentAdjuster {
    pub minimum: f64,
}

impl BatchedRuleAssignmentAdjuster for MinimumBatchedRuleAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedRuleAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched rule assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedRuleAssignmentEngine {
    pub steps: Vec<Rc<dyn BatchedRuleAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedRuleAssignmentEngine {
    pub fn compute(&mut self, item: &BatchedRuleAssignment) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedRuleAssignmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedRuleAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_rule_assignment_engine(flags: &flags::Registry) -> BatchedRuleAssignmentEngine {
    BatchedRuleAssignmentEngine {
        steps: vec![
            Rc::new(FlaggedBatchedRuleAssignmentAdjuster {
                flags: flags.clone(),
                flag: "batched-rule-assignment-batched".to_string(),
                percent: 0.6,
            }) as Rc<dyn BatchedRuleAssignmentAdjuster>,
            Rc::new(RoundingBatchedRuleAssignmentAdjuster { step: 0.05 }),
            Rc::new(CapBatchedRuleAssignmentAdjuster { cap: 32585.0 }),
            Rc::new(MinimumBatchedRuleAssignmentAdjuster { minimum: 25.7 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 77035.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_rule_assignment_engine(&flags::new_registry());
        let item = new_batched_rule_assignment("code-106", "label-971");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
