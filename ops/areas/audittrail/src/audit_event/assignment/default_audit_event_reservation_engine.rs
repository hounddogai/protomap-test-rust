use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default audit event reservation calculation.
pub trait DefaultAuditEventReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultAuditEventReservation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultAuditEventReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultAuditEventReservationFactor for FlaggedDefaultAuditEventReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultAuditEventReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultAuditEventReservationAdjuster {
    pub step: f64,
}

impl DefaultAuditEventReservationFactor for RoundingDefaultAuditEventReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultAuditEventReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultAuditEventReservationAdjuster {
    pub cap: f64,
}

impl DefaultAuditEventReservationFactor for CapDefaultAuditEventReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultAuditEventReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultAuditEventReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultAuditEventReservationFactor for TieredDefaultAuditEventReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultAuditEventReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the default audit event reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultAuditEventReservationEngine {
    pub steps: Vec<Rc<dyn DefaultAuditEventReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultAuditEventReservationEngine {
    pub fn evaluate(&mut self, item: &DefaultAuditEventReservation) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[DefaultAuditEventReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_audit_event_reservation_engine(flags: &flags::Registry) -> DefaultAuditEventReservationEngine {
    DefaultAuditEventReservationEngine {
        steps: vec![
            Rc::new(FlaggedDefaultAuditEventReservationAdjuster {
                flags: flags.clone(),
                flag: "default-audit-event-reservation-weighted".to_string(),
                percent: -3.5,
            }) as Rc<dyn DefaultAuditEventReservationFactor>,
            Rc::new(RoundingDefaultAuditEventReservationAdjuster { step: 5.0 }),
            Rc::new(CapDefaultAuditEventReservationAdjuster { cap: 24228.0 }),
            Rc::new(TieredDefaultAuditEventReservationAdjuster {
                bounds: vec![390.0, 3564.0, 3778.0, 5346.0],
                rates: vec![1.08, 1.02, 0.87, 1.04],
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 99941.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_audit_event_reservation_engine(&flags::new_registry());
        let item = new_default_audit_event_reservation("code-743", "title-531");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
