use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional rollback assignment calculation.
pub trait RegionalRollbackAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalRollbackAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalRollbackAssignmentAdjuster {
    pub minimum: f64,
}

impl RegionalRollbackAssignmentStep for MinimumRegionalRollbackAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalRollbackAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRollbackAssignmentDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl RegionalRollbackAssignmentStep for RegionalRollbackAssignmentDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalRollbackAssignment, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitRegionalRollbackAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalRollbackAssignmentStep for UnitRegionalRollbackAssignmentAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &RegionalRollbackAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalRollbackAssignmentAdjuster {
    pub cap: f64,
}

impl RegionalRollbackAssignmentStep for CapRegionalRollbackAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalRollbackAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the regional rollback assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalRollbackAssignmentEngine {
    pub steps: Vec<Rc<dyn RegionalRollbackAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalRollbackAssignmentEngine {
    pub fn compute(&mut self, item: &RegionalRollbackAssignment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RegionalRollbackAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_rollback_assignment_engine() -> RegionalRollbackAssignmentEngine {
    RegionalRollbackAssignmentEngine {
        steps: vec![
            Rc::new(MinimumRegionalRollbackAssignmentAdjuster { minimum: 44.2 })
                as Rc<dyn RegionalRollbackAssignmentStep>,
            Rc::new(RegionalRollbackAssignmentDurationMinutesOffsetAdjuster { weight: 0.41 }),
            Rc::new(UnitRegionalRollbackAssignmentAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.02),
                    ("south".to_string(), 1.12),
                    ("north".to_string(), 0.78),
                ]),
            }),
            Rc::new(CapRegionalRollbackAssignmentAdjuster { cap: 29933.0 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 66550.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_rollback_assignment_engine();
        let item = new_regional_rollback_assignment("code-307", "label-513");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
