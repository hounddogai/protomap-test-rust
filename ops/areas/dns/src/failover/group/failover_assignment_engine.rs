use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the failover assignment calculation.
pub trait FailoverAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFailoverAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FailoverAssignmentModifier for FlaggedFailoverAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FailoverAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFailoverAssignmentAdjuster {
    pub minimum: f64,
}

impl FailoverAssignmentModifier for MinimumFailoverAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FailoverAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAssignmentCeilingOffsetAdjuster {
    pub weight: f64,
}

impl FailoverAssignmentModifier for FailoverAssignmentCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &FailoverAssignment, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

/// Runs the failover assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverAssignmentEngine {
    pub steps: Vec<Rc<dyn FailoverAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverAssignmentEngine {
    pub fn run(&mut self, item: &FailoverAssignment) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FailoverAssignmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FailoverAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_failover_assignment_engine(flags: &flags::Registry) -> FailoverAssignmentEngine {
    FailoverAssignmentEngine {
        steps: vec![
            Rc::new(FlaggedFailoverAssignmentAdjuster {
                flags: flags.clone(),
                flag: "failover-assignment-strict".to_string(),
                percent: 6.5,
            }) as Rc<dyn FailoverAssignmentModifier>,
            Rc::new(MinimumFailoverAssignmentAdjuster { minimum: 27.7 }),
            Rc::new(FailoverAssignmentCeilingOffsetAdjuster { weight: 1.33 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 26682.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_assignment_engine(&flags::new_registry());
        let item = new_failover_assignment("code-11", "name-18");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
