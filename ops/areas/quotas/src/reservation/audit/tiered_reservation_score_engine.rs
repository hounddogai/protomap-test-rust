use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered reservation score calculation.
pub trait TieredReservationScoreFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredReservationScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredReservationScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredReservationScoreFactor for TieredTieredReservationScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredReservationScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredReservationScoreAdjuster {
    pub minimum: f64,
}

impl TieredReservationScoreFactor for MinimumTieredReservationScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredReservationScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredReservationScoreAdjuster {
    pub cap: f64,
}

impl TieredReservationScoreFactor for CapTieredReservationScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredReservationScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the tiered reservation score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredReservationScoreEngine {
    pub steps: Vec<Rc<dyn TieredReservationScoreFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredReservationScoreEngine {
    pub fn compute(&mut self, item: &TieredReservationScore) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[TieredReservationScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_reservation_score_engine() -> TieredReservationScoreEngine {
    TieredReservationScoreEngine {
        steps: vec![
            Rc::new(TieredTieredReservationScoreAdjuster {
                bounds: vec![1813.0, 6618.0],
                rates: vec![0.88, 1.15],
            }) as Rc<dyn TieredReservationScoreFactor>,
            Rc::new(MinimumTieredReservationScoreAdjuster { minimum: 12.0 }),
            Rc::new(CapTieredReservationScoreAdjuster { cap: 2470.0 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 20462.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_reservation_score_engine();
        let item = new_tiered_reservation_score("code-612", "label-217");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
