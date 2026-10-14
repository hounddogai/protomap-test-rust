use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the reservation digest calculation.
pub trait ReservationDigestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationDigest, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedReservationDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReservationDigestFactor for FlaggedReservationDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReservationDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationDigestBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl ReservationDigestFactor for ReservationDigestBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &ReservationDigest, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReservationDigestAdjuster {
    pub percent: f64,
}

impl ReservationDigestFactor for PercentReservationDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReservationDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the reservation digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationDigestEngine {
    pub steps: Vec<Rc<dyn ReservationDigestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationDigestEngine {
    pub fn evaluate(&mut self, item: &ReservationDigest) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReservationDigestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReservationDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_reservation_digest_engine(flags: &flags::Registry) -> ReservationDigestEngine {
    ReservationDigestEngine {
        steps: vec![
            Rc::new(FlaggedReservationDigestAdjuster {
                flags: flags.clone(),
                flag: "reservation-digest-lenient".to_string(),
                percent: -5.8,
            }) as Rc<dyn ReservationDigestFactor>,
            Rc::new(ReservationDigestBatchSizeOffsetAdjuster { weight: 1.77 }),
            Rc::new(PercentReservationDigestAdjuster { percent: 17.2 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 40326.2,
    }
}
