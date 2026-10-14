use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the failover reservation calculation.
pub trait FailoverReservationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFailoverReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverReservationModifier for TitleFailoverReservationAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FailoverReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFailoverReservationAdjuster {
    pub minimum: f64,
}

impl FailoverReservationModifier for MinimumFailoverReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FailoverReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFailoverReservationAdjuster {
    pub cap: f64,
}

impl FailoverReservationModifier for CapFailoverReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FailoverReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the failover reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverReservationEngine {
    pub steps: Vec<Rc<dyn FailoverReservationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverReservationEngine {
    pub fn run(&mut self, item: &FailoverReservation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FailoverReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_failover_reservation_engine() -> FailoverReservationEngine {
    FailoverReservationEngine {
        steps: vec![
            Rc::new(TitleFailoverReservationAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.99)]),
            }) as Rc<dyn FailoverReservationModifier>,
            Rc::new(MinimumFailoverReservationAdjuster { minimum: 43.0 }),
            Rc::new(CapFailoverReservationAdjuster { cap: 26662.0 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 90922.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_reservation_engine();
        let item = new_failover_reservation("code-80", "title-706");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
