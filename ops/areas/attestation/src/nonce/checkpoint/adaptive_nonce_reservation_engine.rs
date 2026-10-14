use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive nonce reservation calculation.
pub trait AdaptiveNonceReservationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveNonceReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceAdaptiveNonceReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveNonceReservationModifier for NamespaceAdaptiveNonceReservationAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &AdaptiveNonceReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveNonceReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveNonceReservationModifier for FlaggedAdaptiveNonceReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveNonceReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveNonceReservationAdjuster {
    pub minimum: f64,
}

impl AdaptiveNonceReservationModifier for MinimumAdaptiveNonceReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveNonceReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the adaptive nonce reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveNonceReservationEngine {
    pub steps: Vec<Rc<dyn AdaptiveNonceReservationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveNonceReservationEngine {
    pub fn evaluate(&mut self, item: &AdaptiveNonceReservation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveNonceReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_nonce_reservation_engine(flags: &flags::Registry) -> AdaptiveNonceReservationEngine {
    AdaptiveNonceReservationEngine {
        steps: vec![
            Rc::new(NamespaceAdaptiveNonceReservationAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.23),
                    ("central".to_string(), 0.87),
                    ("east".to_string(), 0.81),
                ]),
            }) as Rc<dyn AdaptiveNonceReservationModifier>,
            Rc::new(FlaggedAdaptiveNonceReservationAdjuster {
                flags: flags.clone(),
                flag: "adaptive-nonce-reservation-legacy".to_string(),
                percent: -2.1,
            }),
            Rc::new(MinimumAdaptiveNonceReservationAdjuster { minimum: 16.7 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 77790.0,
    }
}
