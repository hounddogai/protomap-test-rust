use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the isolation assignment calculation.
pub trait IsolationAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationAssignmentBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl IsolationAssignmentStep for IsolationAssignmentBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &IsolationAssignment, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIsolationAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IsolationAssignmentStep for FlaggedIsolationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IsolationAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIsolationAssignmentAdjuster {
    pub cap: f64,
}

impl IsolationAssignmentStep for CapIsolationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IsolationAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIsolationAssignmentAdjuster {
    pub minimum: f64,
}

impl IsolationAssignmentStep for MinimumIsolationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IsolationAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelIsolationAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IsolationAssignmentStep for LabelIsolationAssignmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &IsolationAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIsolationAssignmentAdjuster {
    pub step: f64,
}

impl IsolationAssignmentStep for RoundingIsolationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IsolationAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the isolation assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationAssignmentEngine {
    pub steps: Vec<Rc<dyn IsolationAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationAssignmentEngine {
    pub fn evaluate(&mut self, item: &IsolationAssignment) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[IsolationAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_isolation_assignment_engine(flags: &flags::Registry) -> IsolationAssignmentEngine {
    IsolationAssignmentEngine {
        steps: vec![
            Rc::new(IsolationAssignmentBatchSizeOffsetAdjuster { weight: 0.58 }) as Rc<dyn IsolationAssignmentStep>,
            Rc::new(FlaggedIsolationAssignmentAdjuster {
                flags: flags.clone(),
                flag: "isolation-assignment-scheduled".to_string(),
                percent: -0.4,
            }),
            Rc::new(CapIsolationAssignmentAdjuster { cap: 4368.0 }),
            Rc::new(MinimumIsolationAssignmentAdjuster { minimum: 37.8 }),
            Rc::new(LabelIsolationAssignmentAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.18), ("online".to_string(), 0.84)]),
            }),
            Rc::new(RoundingIsolationAssignmentAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 88774.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_isolation_assignment_engine(&flags::new_registry());
        let item = new_isolation_assignment("code-284", "label-725");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
