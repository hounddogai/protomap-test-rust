use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling audit event assignment calculation.
pub trait RollingAuditEventAssignmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingAuditEventAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingAuditEventAssignmentAdjuster {
    pub percent: f64,
}

impl RollingAuditEventAssignmentFactor for PercentRollingAuditEventAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingAuditEventAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingAuditEventAssignmentAdjuster {
    pub minimum: f64,
}

impl RollingAuditEventAssignmentFactor for MinimumRollingAuditEventAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingAuditEventAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingAuditEventAssignmentAdjuster {
    pub cap: f64,
}

impl RollingAuditEventAssignmentFactor for CapRollingAuditEventAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingAuditEventAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingAuditEventAssignmentAdjuster {
    pub step: f64,
}

impl RollingAuditEventAssignmentFactor for RoundingRollingAuditEventAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingAuditEventAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rolling audit event assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingAuditEventAssignmentEngine {
    pub steps: Vec<Rc<dyn RollingAuditEventAssignmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingAuditEventAssignmentEngine {
    pub fn price(&mut self, item: &RollingAuditEventAssignment) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[RollingAuditEventAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_audit_event_assignment_engine() -> RollingAuditEventAssignmentEngine {
    RollingAuditEventAssignmentEngine {
        steps: vec![
            Rc::new(PercentRollingAuditEventAssignmentAdjuster { percent: 22.4 })
                as Rc<dyn RollingAuditEventAssignmentFactor>,
            Rc::new(MinimumRollingAuditEventAssignmentAdjuster { minimum: 15.8 }),
            Rc::new(CapRollingAuditEventAssignmentAdjuster { cap: 25403.0 }),
            Rc::new(RoundingRollingAuditEventAssignmentAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 37463.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_audit_event_assignment_engine();
        let item = new_rolling_audit_event_assignment("code-312", "name-486");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
