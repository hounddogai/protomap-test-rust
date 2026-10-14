use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient binding reservation calculation.
pub trait LenientBindingReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientBindingReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientBindingReservationAdjuster {
    pub minimum: f64,
}

impl LenientBindingReservationAdjuster for MinimumLenientBindingReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientBindingReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientBindingReservationAdjuster {
    pub cap: f64,
}

impl LenientBindingReservationAdjuster for CapLenientBindingReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientBindingReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameLenientBindingReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientBindingReservationAdjuster for NameLenientBindingReservationAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &LenientBindingReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingReservationSequenceOffsetAdjuster {
    pub weight: f64,
}

impl LenientBindingReservationAdjuster for LenientBindingReservationSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &LenientBindingReservation, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientBindingReservationAdjuster {
    pub step: f64,
}

impl LenientBindingReservationAdjuster for RoundingLenientBindingReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientBindingReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the lenient binding reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientBindingReservationEngine {
    pub steps: Vec<Rc<dyn LenientBindingReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientBindingReservationEngine {
    pub fn evaluate(&mut self, item: &LenientBindingReservation) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientBindingReservationAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientBindingReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_binding_reservation_engine() -> LenientBindingReservationEngine {
    LenientBindingReservationEngine {
        steps: vec![
            Rc::new(MinimumLenientBindingReservationAdjuster { minimum: 33.0 })
                as Rc<dyn LenientBindingReservationAdjuster>,
            Rc::new(CapLenientBindingReservationAdjuster { cap: 48279.0 }),
            Rc::new(NameLenientBindingReservationAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.24)]),
            }),
            Rc::new(LenientBindingReservationSequenceOffsetAdjuster { weight: 0.69 }),
            Rc::new(RoundingLenientBindingReservationAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 53009.5,
    }
}
