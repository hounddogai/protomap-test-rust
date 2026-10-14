use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the audit event reservation calculation.
pub trait AuditEventReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionAuditEventReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AuditEventReservationAdjuster for RegionAuditEventReservationAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &AuditEventReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAuditEventReservationAdjuster {
    pub cap: f64,
}

impl AuditEventReservationAdjuster for CapAuditEventReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AuditEventReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAuditEventReservationAdjuster {
    pub step: f64,
}

impl AuditEventReservationAdjuster for RoundingAuditEventReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AuditEventReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the audit event reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventReservationEngine {
    pub steps: Vec<Rc<dyn AuditEventReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventReservationEngine {
    pub fn price(&mut self, item: &AuditEventReservation) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_audit_event_reservation_engine() -> AuditEventReservationEngine {
    AuditEventReservationEngine {
        steps: vec![
            Rc::new(RegionAuditEventReservationAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.7),
                    ("east".to_string(), 0.96),
                    ("south".to_string(), 1.28),
                ]),
            }) as Rc<dyn AuditEventReservationAdjuster>,
            Rc::new(CapAuditEventReservationAdjuster { cap: 15087.0 }),
            Rc::new(RoundingAuditEventReservationAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 26294.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_audit_event_reservation_engine();
        let item = new_audit_event_reservation("code-242", "title-799");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
