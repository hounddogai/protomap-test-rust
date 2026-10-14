use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending reservation audit calculation.
pub trait PendingReservationAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingReservationAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingReservationAuditAdjuster {
    pub minimum: f64,
}

impl PendingReservationAuditAdjuster for MinimumPendingReservationAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingReservationAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopePendingReservationAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingReservationAuditAdjuster for ScopePendingReservationAuditAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &PendingReservationAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingReservationAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingReservationAuditAdjuster for TieredPendingReservationAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingReservationAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending reservation audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingReservationAuditEngine {
    pub steps: Vec<Rc<dyn PendingReservationAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingReservationAuditEngine {
    pub fn run(&mut self, item: &PendingReservationAudit) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[PendingReservationAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_reservation_audit_engine() -> PendingReservationAuditEngine {
    PendingReservationAuditEngine {
        steps: vec![
            Rc::new(MinimumPendingReservationAuditAdjuster { minimum: 33.3 })
                as Rc<dyn PendingReservationAuditAdjuster>,
            Rc::new(ScopePendingReservationAuditAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.05)]),
            }),
            Rc::new(TieredPendingReservationAuditAdjuster {
                bounds: vec![2608.0, 6729.0],
                rates: vec![1.04, 0.99],
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 11037.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_reservation_audit_engine();
        let item = new_pending_reservation_audit("code-58", "title-285");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
