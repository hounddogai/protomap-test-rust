use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the segment reservation calculation.
pub trait SegmentReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSegmentReservationAdjuster {
    pub percent: f64,
}

impl SegmentReservationAdjuster for PercentSegmentReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SegmentReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSegmentReservationAdjuster {
    pub minimum: f64,
}

impl SegmentReservationAdjuster for MinimumSegmentReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SegmentReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SegmentReservationAdjuster for TieredSegmentReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SegmentReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReservationRetriesOffsetAdjuster {
    pub weight: f64,
}

impl SegmentReservationAdjuster for SegmentReservationRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &SegmentReservation, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleSegmentReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SegmentReservationAdjuster for TitleSegmentReservationAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &SegmentReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the segment reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentReservationEngine {
    pub steps: Vec<Rc<dyn SegmentReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentReservationEngine {
    pub fn compute(&mut self, item: &SegmentReservation) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[SegmentReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_segment_reservation_engine() -> SegmentReservationEngine {
    SegmentReservationEngine {
        steps: vec![
            Rc::new(PercentSegmentReservationAdjuster { percent: 24.2 }) as Rc<dyn SegmentReservationAdjuster>,
            Rc::new(MinimumSegmentReservationAdjuster { minimum: 24.4 }),
            Rc::new(TieredSegmentReservationAdjuster {
                bounds: vec![5436.0, 6172.0],
                rates: vec![0.98, 1.14],
            }),
            Rc::new(SegmentReservationRetriesOffsetAdjuster { weight: 1.14 }),
            Rc::new(TitleSegmentReservationAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.83),
                    ("north".to_string(), 1.3),
                    ("west".to_string(), 0.8),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 88117.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_reservation_engine();
        let item = new_segment_reservation("code-70", "title-77");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
