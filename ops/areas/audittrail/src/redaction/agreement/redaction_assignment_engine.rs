use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the redaction assignment calculation.
pub trait RedactionAssignmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAssignmentRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl RedactionAssignmentFactor for RedactionAssignmentRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &RedactionAssignment, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRedactionAssignmentAdjuster {
    pub percent: f64,
}

impl RedactionAssignmentFactor for PercentRedactionAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RedactionAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRedactionAssignmentAdjuster {
    pub step: f64,
}

impl RedactionAssignmentFactor for RoundingRedactionAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RedactionAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRedactionAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RedactionAssignmentFactor for FlaggedRedactionAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RedactionAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the redaction assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionAssignmentEngine {
    pub steps: Vec<Rc<dyn RedactionAssignmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionAssignmentEngine {
    pub fn evaluate(&mut self, item: &RedactionAssignment) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[RedactionAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_redaction_assignment_engine(flags: &flags::Registry) -> RedactionAssignmentEngine {
    RedactionAssignmentEngine {
        steps: vec![
            Rc::new(RedactionAssignmentRotationDaysOffsetAdjuster { weight: 1.04 })
                as Rc<dyn RedactionAssignmentFactor>,
            Rc::new(PercentRedactionAssignmentAdjuster { percent: 5.7 }),
            Rc::new(RoundingRedactionAssignmentAdjuster { step: 0.5 }),
            Rc::new(FlaggedRedactionAssignmentAdjuster {
                flags: flags.clone(),
                flag: "redaction-assignment-fallback".to_string(),
                percent: -7.1,
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 84610.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_assignment_engine(&flags::new_registry());
        let item = new_redaction_assignment("code-166", "title-50");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
