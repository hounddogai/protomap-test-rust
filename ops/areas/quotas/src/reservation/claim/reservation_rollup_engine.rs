use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the reservation rollup calculation.
pub trait ReservationRollupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReservationRollupAdjuster {
    pub percent: f64,
}

impl ReservationRollupModifier for PercentReservationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReservationRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReservationRollupAdjuster {
    pub cap: f64,
}

impl ReservationRollupModifier for CapReservationRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReservationRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelReservationRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationRollupModifier for LabelReservationRollupAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ReservationRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the reservation rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationRollupEngine {
    pub steps: Vec<Rc<dyn ReservationRollupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationRollupEngine {
    pub fn price(&mut self, item: &ReservationRollup) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReservationRollupModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReservationRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_reservation_rollup_engine() -> ReservationRollupEngine {
    ReservationRollupEngine {
        steps: vec![
            Rc::new(PercentReservationRollupAdjuster { percent: -12.0 }) as Rc<dyn ReservationRollupModifier>,
            Rc::new(CapReservationRollupAdjuster { cap: 16864.0 }),
            Rc::new(LabelReservationRollupAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.22),
                    ("central".to_string(), 0.99),
                    ("online".to_string(), 1.15),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 88209.8,
    }
}
