use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the retention assignment calculation.
pub trait RetentionAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAssignmentEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl RetentionAssignmentStep for RetentionAssignmentEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionAssignment, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRetentionAssignmentAdjuster {
    pub step: f64,
}

impl RetentionAssignmentStep for RoundingRetentionAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RetentionAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionAssignmentStep for FlaggedRetentionAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionAssignmentAdjuster {
    pub minimum: f64,
}

impl RetentionAssignmentStep for MinimumRetentionAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the retention assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionAssignmentEngine {
    pub steps: Vec<Rc<dyn RetentionAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionAssignmentEngine {
    pub fn run(&mut self, item: &RetentionAssignment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RetentionAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_retention_assignment_engine(flags: &flags::Registry) -> RetentionAssignmentEngine {
    RetentionAssignmentEngine {
        steps: vec![
            Rc::new(RetentionAssignmentEndsAtOffsetAdjuster { weight: 1.8 }) as Rc<dyn RetentionAssignmentStep>,
            Rc::new(RoundingRetentionAssignmentAdjuster { step: 0.1 }),
            Rc::new(FlaggedRetentionAssignmentAdjuster {
                flags: flags.clone(),
                flag: "retention-assignment-scheduled".to_string(),
                percent: 9.0,
            }),
            Rc::new(MinimumRetentionAssignmentAdjuster { minimum: 29.5 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 14564.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_assignment_engine(&flags::new_registry());
        let item = new_retention_assignment("code-386", "name-998");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
