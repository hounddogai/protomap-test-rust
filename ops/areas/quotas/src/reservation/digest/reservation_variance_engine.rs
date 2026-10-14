use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the reservation variance calculation.
pub trait ReservationVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReservationVarianceStep for TieredReservationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReservationVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReservationVarianceAdjuster {
    pub step: f64,
}

impl ReservationVarianceStep for RoundingReservationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReservationVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryReservationVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationVarianceStep for CountryReservationVarianceAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &ReservationVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationVarianceLimitOffsetAdjuster {
    pub weight: f64,
}

impl ReservationVarianceStep for ReservationVarianceLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &ReservationVariance, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

/// Runs the reservation variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationVarianceEngine {
    pub steps: Vec<Rc<dyn ReservationVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationVarianceEngine {
    pub fn run(&mut self, item: &ReservationVariance) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReservationVarianceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReservationVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_reservation_variance_engine() -> ReservationVarianceEngine {
    ReservationVarianceEngine {
        steps: vec![
            Rc::new(TieredReservationVarianceAdjuster {
                bounds: vec![1440.0, 3810.0],
                rates: vec![0.97, 1.18],
            }) as Rc<dyn ReservationVarianceStep>,
            Rc::new(RoundingReservationVarianceAdjuster { step: 0.1 }),
            Rc::new(CountryReservationVarianceAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.05),
                    ("online".to_string(), 0.76),
                    ("west".to_string(), 1.28),
                ]),
            }),
            Rc::new(ReservationVarianceLimitOffsetAdjuster { weight: 1.28 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 12884.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_variance_engine();
        let item = new_reservation_variance("code-375", "label-251");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
