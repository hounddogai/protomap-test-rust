use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the primary reservation variance calculation.
pub trait PrimaryReservationVarianceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryReservationVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryReservationVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryReservationVarianceAdjuster for TieredPrimaryReservationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryReservationVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePrimaryReservationVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryReservationVarianceAdjuster for TitlePrimaryReservationVarianceAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PrimaryReservationVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryReservationVarianceAdjuster {
    pub minimum: f64,
}

impl PrimaryReservationVarianceAdjuster for MinimumPrimaryReservationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryReservationVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the primary reservation variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryReservationVarianceEngine {
    pub steps: Vec<Rc<dyn PrimaryReservationVarianceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryReservationVarianceEngine {
    pub fn evaluate(&mut self, item: &PrimaryReservationVariance) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryReservationVarianceAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryReservationVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_reservation_variance_engine() -> PrimaryReservationVarianceEngine {
    PrimaryReservationVarianceEngine {
        steps: vec![
            Rc::new(TieredPrimaryReservationVarianceAdjuster {
                bounds: vec![620.0, 2166.0, 8441.0, 9388.0],
                rates: vec![1.01, 0.91, 1.03, 1.0],
            }) as Rc<dyn PrimaryReservationVarianceAdjuster>,
            Rc::new(TitlePrimaryReservationVarianceAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.38)]),
            }),
            Rc::new(MinimumPrimaryReservationVarianceAdjuster { minimum: 19.5 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 7018.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_reservation_variance_engine();
        let item = new_primary_reservation_variance("code-248", "title-379");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
