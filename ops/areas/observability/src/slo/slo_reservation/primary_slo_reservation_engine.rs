use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary slo reservation calculation.
pub trait PrimarySloReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimarySloReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimarySloReservationAdjuster {
    pub percent: f64,
}

impl PrimarySloReservationAdjuster for PercentPrimarySloReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimarySloReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloReservationCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl PrimarySloReservationAdjuster for PrimarySloReservationCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &PrimarySloReservation, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimarySloReservationAdjuster {
    pub step: f64,
}

impl PrimarySloReservationAdjuster for RoundingPrimarySloReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimarySloReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimarySloReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimarySloReservationAdjuster for TieredPrimarySloReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimarySloReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimarySloReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimarySloReservationAdjuster for FlaggedPrimarySloReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimarySloReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary slo reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimarySloReservationEngine {
    pub steps: Vec<Rc<dyn PrimarySloReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimarySloReservationEngine {
    pub fn price(&mut self, item: &PrimarySloReservation) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimarySloReservationAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimarySloReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_slo_reservation_engine(flags: &flags::Registry) -> PrimarySloReservationEngine {
    PrimarySloReservationEngine {
        steps: vec![
            Rc::new(PercentPrimarySloReservationAdjuster { percent: -1.7 }) as Rc<dyn PrimarySloReservationAdjuster>,
            Rc::new(PrimarySloReservationCreatedAtOffsetAdjuster { weight: 1.27 }),
            Rc::new(RoundingPrimarySloReservationAdjuster { step: 5.0 }),
            Rc::new(TieredPrimarySloReservationAdjuster {
                bounds: vec![3673.0, 4242.0, 4801.0],
                rates: vec![0.86, 0.89, 0.87],
            }),
            Rc::new(FlaggedPrimarySloReservationAdjuster {
                flags: flags.clone(),
                flag: "primary-slo-reservation-rolling".to_string(),
                percent: 0.8,
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 29318.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_slo_reservation_engine(&flags::new_registry());
        let item = new_primary_slo_reservation("code-610", "label-858");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
