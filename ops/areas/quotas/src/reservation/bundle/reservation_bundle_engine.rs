use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the reservation bundle calculation.
pub trait ReservationBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitReservationBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationBundleStep for UnitReservationBundleAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &ReservationBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReservationBundleStep for TieredReservationBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReservationBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReservationBundleAdjuster {
    pub cap: f64,
}

impl ReservationBundleStep for CapReservationBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReservationBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReservationBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReservationBundleStep for FlaggedReservationBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReservationBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReservationBundleAdjuster {
    pub percent: f64,
}

impl ReservationBundleStep for PercentReservationBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReservationBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the reservation bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationBundleEngine {
    pub steps: Vec<Rc<dyn ReservationBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationBundleEngine {
    pub fn evaluate(&mut self, item: &ReservationBundle) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[ReservationBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_reservation_bundle_engine(flags: &flags::Registry) -> ReservationBundleEngine {
    ReservationBundleEngine {
        steps: vec![
            Rc::new(UnitReservationBundleAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.36),
                    ("east".to_string(), 1.39),
                    ("north".to_string(), 1.33),
                ]),
            }) as Rc<dyn ReservationBundleStep>,
            Rc::new(TieredReservationBundleAdjuster {
                bounds: vec![2945.0, 9085.0],
                rates: vec![1.17, 0.92],
            }),
            Rc::new(CapReservationBundleAdjuster { cap: 44652.0 }),
            Rc::new(FlaggedReservationBundleAdjuster {
                flags: flags.clone(),
                flag: "reservation-bundle-manual".to_string(),
                percent: -8.7,
            }),
            Rc::new(PercentReservationBundleAdjuster { percent: 21.6 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 52305.5,
    }
}
