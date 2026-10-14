use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient segment reservation calculation.
pub trait LenientSegmentReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientSegmentReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientSegmentReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientSegmentReservationFactor for TieredLenientSegmentReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientSegmentReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSegmentReservationUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl LenientSegmentReservationFactor for LenientSegmentReservationUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &LenientSegmentReservation, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientSegmentReservationAdjuster {
    pub minimum: f64,
}

impl LenientSegmentReservationFactor for MinimumLenientSegmentReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientSegmentReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientSegmentReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientSegmentReservationFactor for FlaggedLenientSegmentReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientSegmentReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientSegmentReservationAdjuster {
    pub cap: f64,
}

impl LenientSegmentReservationFactor for CapLenientSegmentReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientSegmentReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the lenient segment reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientSegmentReservationEngine {
    pub steps: Vec<Rc<dyn LenientSegmentReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientSegmentReservationEngine {
    pub fn compute(&mut self, item: &LenientSegmentReservation) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientSegmentReservationFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientSegmentReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_segment_reservation_engine(flags: &flags::Registry) -> LenientSegmentReservationEngine {
    LenientSegmentReservationEngine {
        steps: vec![
            Rc::new(TieredLenientSegmentReservationAdjuster {
                bounds: vec![4629.0, 5007.0, 5708.0],
                rates: vec![0.81, 1.17, 1.18],
            }) as Rc<dyn LenientSegmentReservationFactor>,
            Rc::new(LenientSegmentReservationUpdatedAtOffsetAdjuster { weight: 0.41 }),
            Rc::new(MinimumLenientSegmentReservationAdjuster { minimum: 42.0 }),
            Rc::new(FlaggedLenientSegmentReservationAdjuster {
                flags: flags.clone(),
                flag: "lenient-segment-reservation-adaptive".to_string(),
                percent: 0.5,
            }),
            Rc::new(CapLenientSegmentReservationAdjuster { cap: 36568.0 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 1160.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_segment_reservation_engine(&flags::new_registry());
        let item = new_lenient_segment_reservation("code-496", "name-999");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
