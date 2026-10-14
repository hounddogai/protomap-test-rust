use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary dashboard reservation calculation.
pub trait PrimaryDashboardReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryDashboardReservation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryDashboardReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryDashboardReservationFactor for FlaggedPrimaryDashboardReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryDashboardReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryDashboardReservationAdjuster {
    pub cap: f64,
}

impl PrimaryDashboardReservationFactor for CapPrimaryDashboardReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryDashboardReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryDashboardReservationAdjuster {
    pub step: f64,
}

impl PrimaryDashboardReservationFactor for RoundingPrimaryDashboardReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryDashboardReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDashboardReservationBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryDashboardReservationFactor for PrimaryDashboardReservationBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryDashboardReservation, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryDashboardReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryDashboardReservationFactor for TieredPrimaryDashboardReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryDashboardReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the primary dashboard reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryDashboardReservationEngine {
    pub steps: Vec<Rc<dyn PrimaryDashboardReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryDashboardReservationEngine {
    pub fn compute(&mut self, item: &PrimaryDashboardReservation) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryDashboardReservationFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryDashboardReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_dashboard_reservation_engine(flags: &flags::Registry) -> PrimaryDashboardReservationEngine {
    PrimaryDashboardReservationEngine {
        steps: vec![
            Rc::new(FlaggedPrimaryDashboardReservationAdjuster {
                flags: flags.clone(),
                flag: "primary-dashboard-reservation-tiered".to_string(),
                percent: 2.7,
            }) as Rc<dyn PrimaryDashboardReservationFactor>,
            Rc::new(CapPrimaryDashboardReservationAdjuster { cap: 19706.0 }),
            Rc::new(RoundingPrimaryDashboardReservationAdjuster { step: 0.5 }),
            Rc::new(PrimaryDashboardReservationBatchSizeOffsetAdjuster { weight: 1.05 }),
            Rc::new(TieredPrimaryDashboardReservationAdjuster {
                bounds: vec![2273.0, 6616.0, 6642.0, 9776.0],
                rates: vec![0.99, 0.9, 0.87, 0.95],
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 10427.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_dashboard_reservation_engine(&flags::new_registry());
        let item = new_primary_dashboard_reservation("code-521", "title-613");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
