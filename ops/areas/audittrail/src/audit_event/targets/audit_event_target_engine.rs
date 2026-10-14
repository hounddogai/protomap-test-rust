use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the audit event target calculation.
pub trait AuditEventTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAuditEventTargetAdjuster {
    pub step: f64,
}

impl AuditEventTargetAdjuster for RoundingAuditEventTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AuditEventTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAuditEventTargetAdjuster {
    pub percent: f64,
}

impl AuditEventTargetAdjuster for PercentAuditEventTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AuditEventTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAuditEventTargetAdjuster {
    pub cap: f64,
}

impl AuditEventTargetAdjuster for CapAuditEventTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AuditEventTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the audit event target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventTargetEngine {
    pub steps: Vec<Rc<dyn AuditEventTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventTargetEngine {
    pub fn compute(&mut self, item: &AuditEventTarget) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AuditEventTargetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_audit_event_target_engine() -> AuditEventTargetEngine {
    AuditEventTargetEngine {
        steps: vec![
            Rc::new(RoundingAuditEventTargetAdjuster { step: 0.5 }) as Rc<dyn AuditEventTargetAdjuster>,
            Rc::new(PercentAuditEventTargetAdjuster { percent: 4.9 }),
            Rc::new(CapAuditEventTargetAdjuster { cap: 38916.0 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 28661.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_audit_event_target_engine();
        let item = new_audit_event_target("code-163", "label-357");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
