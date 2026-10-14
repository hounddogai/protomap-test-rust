use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the runbook assignment calculation.
pub trait RunbookAssignmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRunbookAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RunbookAssignmentFactor for FlaggedRunbookAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RunbookAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookAssignmentAdjuster {
    pub percent: f64,
}

impl RunbookAssignmentFactor for PercentRunbookAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookAssignmentAdjuster {
    pub cap: f64,
}

impl RunbookAssignmentFactor for CapRunbookAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRunbookAssignmentAdjuster {
    pub minimum: f64,
}

impl RunbookAssignmentFactor for MinimumRunbookAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RunbookAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the runbook assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookAssignmentEngine {
    pub steps: Vec<Rc<dyn RunbookAssignmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookAssignmentEngine {
    pub fn price(&mut self, item: &RunbookAssignment) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RunbookAssignmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RunbookAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_runbook_assignment_engine(flags: &flags::Registry) -> RunbookAssignmentEngine {
    RunbookAssignmentEngine {
        steps: vec![
            Rc::new(FlaggedRunbookAssignmentAdjuster {
                flags: flags.clone(),
                flag: "runbook-assignment-adaptive".to_string(),
                percent: -8.4,
            }) as Rc<dyn RunbookAssignmentFactor>,
            Rc::new(PercentRunbookAssignmentAdjuster { percent: -17.4 }),
            Rc::new(CapRunbookAssignmentAdjuster { cap: 44629.0 }),
            Rc::new(MinimumRunbookAssignmentAdjuster { minimum: 43.5 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 47250.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_assignment_engine(&flags::new_registry());
        let item = new_runbook_assignment("code-937", "title-46");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
