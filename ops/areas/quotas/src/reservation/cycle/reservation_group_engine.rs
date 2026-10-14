use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the reservation group calculation.
pub trait ReservationGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReservationGroupStep for TieredReservationGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReservationGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReservationGroupAdjuster {
    pub percent: f64,
}

impl ReservationGroupStep for PercentReservationGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReservationGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReservationGroupAdjuster {
    pub step: f64,
}

impl ReservationGroupStep for RoundingReservationGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReservationGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReservationGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReservationGroupStep for FlaggedReservationGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReservationGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReservationGroupAdjuster {
    pub cap: f64,
}

impl ReservationGroupStep for CapReservationGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReservationGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the reservation group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationGroupEngine {
    pub steps: Vec<Rc<dyn ReservationGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationGroupEngine {
    pub fn run(&mut self, item: &ReservationGroup) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReservationGroupStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReservationGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_reservation_group_engine(flags: &flags::Registry) -> ReservationGroupEngine {
    ReservationGroupEngine {
        steps: vec![
            Rc::new(TieredReservationGroupAdjuster {
                bounds: vec![5555.0, 6276.0, 6574.0, 9303.0],
                rates: vec![0.87, 1.08, 0.95, 0.95],
            }) as Rc<dyn ReservationGroupStep>,
            Rc::new(PercentReservationGroupAdjuster { percent: 21.6 }),
            Rc::new(RoundingReservationGroupAdjuster { step: 0.05 }),
            Rc::new(FlaggedReservationGroupAdjuster {
                flags: flags.clone(),
                flag: "reservation-group-batched".to_string(),
                percent: -7.1,
            }),
            Rc::new(CapReservationGroupAdjuster { cap: 48981.0 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 80978.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_group_engine(&flags::new_registry());
        let item = new_reservation_group("code-407", "label-297");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
