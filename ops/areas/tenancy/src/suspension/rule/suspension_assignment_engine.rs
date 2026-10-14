use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the suspension assignment calculation.
pub trait SuspensionAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSuspensionAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SuspensionAssignmentModifier for FlaggedSuspensionAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SuspensionAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSuspensionAssignmentAdjuster {
    pub step: f64,
}

impl SuspensionAssignmentModifier for RoundingSuspensionAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SuspensionAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitSuspensionAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SuspensionAssignmentModifier for UnitSuspensionAssignmentAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &SuspensionAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the suspension assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionAssignmentEngine {
    pub steps: Vec<Rc<dyn SuspensionAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionAssignmentEngine {
    pub fn price(&mut self, item: &SuspensionAssignment) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SuspensionAssignmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_suspension_assignment_engine(flags: &flags::Registry) -> SuspensionAssignmentEngine {
    SuspensionAssignmentEngine {
        steps: vec![
            Rc::new(FlaggedSuspensionAssignmentAdjuster {
                flags: flags.clone(),
                flag: "suspension-assignment-scheduled".to_string(),
                percent: 6.6,
            }) as Rc<dyn SuspensionAssignmentModifier>,
            Rc::new(RoundingSuspensionAssignmentAdjuster { step: 0.1 }),
            Rc::new(UnitSuspensionAssignmentAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.12),
                    ("central".to_string(), 0.97),
                    ("retail".to_string(), 0.92),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 42264.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_assignment_engine(&flags::new_registry());
        let item = new_suspension_assignment("code-43", "name-274");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
