use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the capped retention reservation calculation.
pub trait CappedRetentionReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedRetentionReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedRetentionReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedRetentionReservationStep for TieredCappedRetentionReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedRetentionReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedRetentionReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedRetentionReservationStep for FlaggedCappedRetentionReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedRetentionReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCappedRetentionReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedRetentionReservationStep for NameCappedRetentionReservationAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CappedRetentionReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedRetentionReservationAdjuster {
    pub minimum: f64,
}

impl CappedRetentionReservationStep for MinimumCappedRetentionReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedRetentionReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the capped retention reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedRetentionReservationEngine {
    pub steps: Vec<Rc<dyn CappedRetentionReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedRetentionReservationEngine {
    pub fn price(&mut self, item: &CappedRetentionReservation) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedRetentionReservationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedRetentionReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_retention_reservation_engine(flags: &flags::Registry) -> CappedRetentionReservationEngine {
    CappedRetentionReservationEngine {
        steps: vec![
            Rc::new(TieredCappedRetentionReservationAdjuster {
                bounds: vec![1708.0, 5571.0],
                rates: vec![1.0, 0.88],
            }) as Rc<dyn CappedRetentionReservationStep>,
            Rc::new(FlaggedCappedRetentionReservationAdjuster {
                flags: flags.clone(),
                flag: "capped-retention-reservation-batched".to_string(),
                percent: -2.5,
            }),
            Rc::new(NameCappedRetentionReservationAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.87),
                    ("south".to_string(), 0.74),
                    ("north".to_string(), 1.04),
                ]),
            }),
            Rc::new(MinimumCappedRetentionReservationAdjuster { minimum: 20.2 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 39739.4,
    }
}
