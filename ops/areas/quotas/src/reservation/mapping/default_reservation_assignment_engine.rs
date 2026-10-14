use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default reservation assignment calculation.
pub trait DefaultReservationAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultReservationAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultReservationAssignmentAdjuster {
    pub minimum: f64,
}

impl DefaultReservationAssignmentAdjuster for MinimumDefaultReservationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultReservationAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultReservationAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultReservationAssignmentAdjuster for FlaggedDefaultReservationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultReservationAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationAssignmentRateOffsetAdjuster {
    pub weight: f64,
}

impl DefaultReservationAssignmentAdjuster for DefaultReservationAssignmentRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultReservationAssignment, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

/// Runs the default reservation assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultReservationAssignmentEngine {
    pub steps: Vec<Rc<dyn DefaultReservationAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultReservationAssignmentEngine {
    pub fn price(&mut self, item: &DefaultReservationAssignment) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[DefaultReservationAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_reservation_assignment_engine(flags: &flags::Registry) -> DefaultReservationAssignmentEngine {
    DefaultReservationAssignmentEngine {
        steps: vec![
            Rc::new(MinimumDefaultReservationAssignmentAdjuster { minimum: 2.5 })
                as Rc<dyn DefaultReservationAssignmentAdjuster>,
            Rc::new(FlaggedDefaultReservationAssignmentAdjuster {
                flags: flags.clone(),
                flag: "default-reservation-assignment-weighted".to_string(),
                percent: -5.2,
            }),
            Rc::new(DefaultReservationAssignmentRateOffsetAdjuster { weight: 1.19 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 4239.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_reservation_assignment_engine(&flags::new_registry());
        let item = new_default_reservation_assignment("code-321", "name-693");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
