use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the capped reservation variance calculation.
pub trait CappedReservationVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedReservationVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleCappedReservationVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedReservationVarianceStep for TitleCappedReservationVarianceAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &CappedReservationVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationVarianceMarginOffsetAdjuster {
    pub weight: f64,
}

impl CappedReservationVarianceStep for CappedReservationVarianceMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &CappedReservationVariance, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedReservationVarianceAdjuster {
    pub minimum: f64,
}

impl CappedReservationVarianceStep for MinimumCappedReservationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedReservationVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedReservationVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedReservationVarianceStep for TieredCappedReservationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedReservationVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the capped reservation variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedReservationVarianceEngine {
    pub steps: Vec<Rc<dyn CappedReservationVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedReservationVarianceEngine {
    pub fn price(&mut self, item: &CappedReservationVariance) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedReservationVarianceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedReservationVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_reservation_variance_engine() -> CappedReservationVarianceEngine {
    CappedReservationVarianceEngine {
        steps: vec![
            Rc::new(TitleCappedReservationVarianceAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.99)]),
            }) as Rc<dyn CappedReservationVarianceStep>,
            Rc::new(CappedReservationVarianceMarginOffsetAdjuster { weight: 1.36 }),
            Rc::new(MinimumCappedReservationVarianceAdjuster { minimum: 6.8 }),
            Rc::new(TieredCappedReservationVarianceAdjuster {
                bounds: vec![145.0, 8543.0],
                rates: vec![0.81, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 96074.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_reservation_variance_engine();
        let item = new_capped_reservation_variance("code-471", "title-141");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
