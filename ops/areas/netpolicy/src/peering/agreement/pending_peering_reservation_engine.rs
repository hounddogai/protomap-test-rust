use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending peering reservation calculation.
pub trait PendingPeeringReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingPeeringReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingPeeringReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingPeeringReservationStep for TieredPendingPeeringReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingPeeringReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryPendingPeeringReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingPeeringReservationStep for CategoryPendingPeeringReservationAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &PendingPeeringReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingPeeringReservationAdjuster {
    pub step: f64,
}

impl PendingPeeringReservationStep for RoundingPendingPeeringReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingPeeringReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingPeeringReservationAdjuster {
    pub cap: f64,
}

impl PendingPeeringReservationStep for CapPendingPeeringReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingPeeringReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingPeeringReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingPeeringReservationStep for FlaggedPendingPeeringReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingPeeringReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending peering reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingPeeringReservationEngine {
    pub steps: Vec<Rc<dyn PendingPeeringReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingPeeringReservationEngine {
    pub fn evaluate(&mut self, item: &PendingPeeringReservation) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingPeeringReservationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingPeeringReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_peering_reservation_engine(flags: &flags::Registry) -> PendingPeeringReservationEngine {
    PendingPeeringReservationEngine {
        steps: vec![
            Rc::new(TieredPendingPeeringReservationAdjuster {
                bounds: vec![2130.0, 3876.0, 6315.0, 9848.0],
                rates: vec![0.91, 0.95, 1.12, 0.92],
            }) as Rc<dyn PendingPeeringReservationStep>,
            Rc::new(CategoryPendingPeeringReservationAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.12)]),
            }),
            Rc::new(RoundingPendingPeeringReservationAdjuster { step: 5.0 }),
            Rc::new(CapPendingPeeringReservationAdjuster { cap: 37344.0 }),
            Rc::new(FlaggedPendingPeeringReservationAdjuster {
                flags: flags.clone(),
                flag: "pending-peering-reservation-legacy".to_string(),
                percent: -9.2,
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 1170.7,
    }
}
