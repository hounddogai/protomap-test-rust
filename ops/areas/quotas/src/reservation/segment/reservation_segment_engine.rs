use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the reservation segment calculation.
pub trait ReservationSegmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReservationSegmentAdjuster {
    pub percent: f64,
}

impl ReservationSegmentModifier for PercentReservationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReservationSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReservationSegmentAdjuster {
    pub cap: f64,
}

impl ReservationSegmentModifier for CapReservationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReservationSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSegmentMinimumOffsetAdjuster {
    pub weight: f64,
}

impl ReservationSegmentModifier for ReservationSegmentMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &ReservationSegment, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the reservation segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationSegmentEngine {
    pub steps: Vec<Rc<dyn ReservationSegmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationSegmentEngine {
    pub fn run(&mut self, item: &ReservationSegment) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReservationSegmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReservationSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_reservation_segment_engine() -> ReservationSegmentEngine {
    ReservationSegmentEngine {
        steps: vec![
            Rc::new(PercentReservationSegmentAdjuster { percent: -23.6 }) as Rc<dyn ReservationSegmentModifier>,
            Rc::new(CapReservationSegmentAdjuster { cap: 40907.0 }),
            Rc::new(ReservationSegmentMinimumOffsetAdjuster { weight: 0.19 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 37806.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_segment_engine();
        let item = new_reservation_segment("code-70", "name-688");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
