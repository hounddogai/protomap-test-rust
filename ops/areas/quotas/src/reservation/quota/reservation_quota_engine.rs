use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the reservation quota calculation.
pub trait ReservationQuotaStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationQuotaMaximumOffsetAdjuster {
    pub weight: f64,
}

impl ReservationQuotaStep for ReservationQuotaMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &ReservationQuota, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReservationQuotaAdjuster {
    pub cap: f64,
}

impl ReservationQuotaStep for CapReservationQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReservationQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReservationQuotaAdjuster {
    pub percent: f64,
}

impl ReservationQuotaStep for PercentReservationQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReservationQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReservationQuotaAdjuster {
    pub minimum: f64,
}

impl ReservationQuotaStep for MinimumReservationQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReservationQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the reservation quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationQuotaEngine {
    pub steps: Vec<Rc<dyn ReservationQuotaStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationQuotaEngine {
    pub fn evaluate(&mut self, item: &ReservationQuota) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[ReservationQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_reservation_quota_engine() -> ReservationQuotaEngine {
    ReservationQuotaEngine {
        steps: vec![
            Rc::new(ReservationQuotaMaximumOffsetAdjuster { weight: 0.63 }) as Rc<dyn ReservationQuotaStep>,
            Rc::new(CapReservationQuotaAdjuster { cap: 15812.0 }),
            Rc::new(PercentReservationQuotaAdjuster { percent: 23.6 }),
            Rc::new(MinimumReservationQuotaAdjuster { minimum: 35.6 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 92667.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_quota_engine();
        let item = new_reservation_quota("code-715", "label-242");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
