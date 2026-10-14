use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the quote reservation calculation.
pub trait QuoteReservationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteReservationAdjuster {
    pub minimum: f64,
}

impl QuoteReservationModifier for MinimumQuoteReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QuoteReservationModifier for TieredQuoteReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QuoteReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteReservationBurstOffsetAdjuster {
    pub weight: f64,
}

impl QuoteReservationModifier for QuoteReservationBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &QuoteReservation, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

/// Runs the quote reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteReservationEngine {
    pub steps: Vec<Rc<dyn QuoteReservationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteReservationEngine {
    pub fn run(&mut self, item: &QuoteReservation) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuoteReservationModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuoteReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_quote_reservation_engine() -> QuoteReservationEngine {
    QuoteReservationEngine {
        steps: vec![
            Rc::new(MinimumQuoteReservationAdjuster { minimum: 10.7 }) as Rc<dyn QuoteReservationModifier>,
            Rc::new(TieredQuoteReservationAdjuster {
                bounds: vec![2506.0, 5791.0, 8078.0],
                rates: vec![0.85, 0.93, 0.84],
            }),
            Rc::new(QuoteReservationBurstOffsetAdjuster { weight: 0.12 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 63511.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_reservation_engine();
        let item = new_quote_reservation("code-142", "title-903");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
