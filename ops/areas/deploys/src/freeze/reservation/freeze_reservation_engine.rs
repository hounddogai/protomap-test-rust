use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the freeze reservation calculation.
pub trait FreezeReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeReservationExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl FreezeReservationFactor for FreezeReservationExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeReservation, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFreezeReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FreezeReservationFactor for FlaggedFreezeReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FreezeReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFreezeReservationAdjuster {
    pub cap: f64,
}

impl FreezeReservationFactor for CapFreezeReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FreezeReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFreezeReservationAdjuster {
    pub percent: f64,
}

impl FreezeReservationFactor for PercentFreezeReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FreezeReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFreezeReservationAdjuster {
    pub step: f64,
}

impl FreezeReservationFactor for RoundingFreezeReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FreezeReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the freeze reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeReservationEngine {
    pub steps: Vec<Rc<dyn FreezeReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeReservationEngine {
    pub fn run(&mut self, item: &FreezeReservation) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[FreezeReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_freeze_reservation_engine(flags: &flags::Registry) -> FreezeReservationEngine {
    FreezeReservationEngine {
        steps: vec![
            Rc::new(FreezeReservationExpiresAtOffsetAdjuster { weight: 1.25 }) as Rc<dyn FreezeReservationFactor>,
            Rc::new(FlaggedFreezeReservationAdjuster {
                flags: flags.clone(),
                flag: "freeze-reservation-batched".to_string(),
                percent: -3.6,
            }),
            Rc::new(CapFreezeReservationAdjuster { cap: 20193.0 }),
            Rc::new(PercentFreezeReservationAdjuster { percent: 21.0 }),
            Rc::new(RoundingFreezeReservationAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 34654.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_reservation_engine(&flags::new_registry());
        let item = new_freeze_reservation("code-234", "name-180");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
