use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the canary reservation calculation.
pub trait CanaryReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelCanaryReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CanaryReservationStep for LabelCanaryReservationAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &CanaryReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCanaryReservationAdjuster {
    pub percent: f64,
}

impl CanaryReservationStep for PercentCanaryReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CanaryReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryReservationWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl CanaryReservationStep for CanaryReservationWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &CanaryReservation, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

/// Runs the canary reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryReservationEngine {
    pub steps: Vec<Rc<dyn CanaryReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryReservationEngine {
    pub fn evaluate(&mut self, item: &CanaryReservation) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[CanaryReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_canary_reservation_engine() -> CanaryReservationEngine {
    CanaryReservationEngine {
        steps: vec![
            Rc::new(LabelCanaryReservationAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.15)]),
            }) as Rc<dyn CanaryReservationStep>,
            Rc::new(PercentCanaryReservationAdjuster { percent: 3.9 }),
            Rc::new(CanaryReservationWindowDaysOffsetAdjuster { weight: 0.33 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 9920.9,
    }
}
