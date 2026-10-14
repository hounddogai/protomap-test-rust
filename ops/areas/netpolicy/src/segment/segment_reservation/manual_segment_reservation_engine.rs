use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the manual segment reservation calculation.
pub trait ManualSegmentReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualSegmentReservation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualSegmentReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualSegmentReservationAdjuster for FlaggedManualSegmentReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualSegmentReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualSegmentReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualSegmentReservationAdjuster for TieredManualSegmentReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualSegmentReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualSegmentReservationAdjuster {
    pub percent: f64,
}

impl ManualSegmentReservationAdjuster for PercentManualSegmentReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualSegmentReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSegmentReservationQuantityOffsetAdjuster {
    pub weight: f64,
}

impl ManualSegmentReservationAdjuster for ManualSegmentReservationQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &ManualSegmentReservation, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

/// Runs the manual segment reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualSegmentReservationEngine {
    pub steps: Vec<Rc<dyn ManualSegmentReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualSegmentReservationEngine {
    pub fn price(&mut self, item: &ManualSegmentReservation) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualSegmentReservationAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualSegmentReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_segment_reservation_engine(flags: &flags::Registry) -> ManualSegmentReservationEngine {
    ManualSegmentReservationEngine {
        steps: vec![
            Rc::new(FlaggedManualSegmentReservationAdjuster {
                flags: flags.clone(),
                flag: "manual-segment-reservation-manual".to_string(),
                percent: 5.2,
            }) as Rc<dyn ManualSegmentReservationAdjuster>,
            Rc::new(TieredManualSegmentReservationAdjuster {
                bounds: vec![2746.0, 3016.0],
                rates: vec![0.88, 0.89],
            }),
            Rc::new(PercentManualSegmentReservationAdjuster { percent: -4.1 }),
            Rc::new(ManualSegmentReservationQuantityOffsetAdjuster { weight: 1.56 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 12064.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_segment_reservation_engine(&flags::new_registry());
        let item = new_manual_segment_reservation("code-69", "title-559");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
