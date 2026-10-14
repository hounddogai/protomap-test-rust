use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the global reservation variance calculation.
pub trait GlobalReservationVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalReservationVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationVarianceCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl GlobalReservationVarianceStep for GlobalReservationVarianceCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalReservationVariance, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalReservationVarianceAdjuster {
    pub minimum: f64,
}

impl GlobalReservationVarianceStep for MinimumGlobalReservationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalReservationVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalReservationVarianceAdjuster {
    pub percent: f64,
}

impl GlobalReservationVarianceStep for PercentGlobalReservationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalReservationVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global reservation variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalReservationVarianceEngine {
    pub steps: Vec<Rc<dyn GlobalReservationVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalReservationVarianceEngine {
    pub fn compute(&mut self, item: &GlobalReservationVariance) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalReservationVarianceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalReservationVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_reservation_variance_engine() -> GlobalReservationVarianceEngine {
    GlobalReservationVarianceEngine {
        steps: vec![
            Rc::new(GlobalReservationVarianceCreatedAtOffsetAdjuster { weight: 1.56 })
                as Rc<dyn GlobalReservationVarianceStep>,
            Rc::new(MinimumGlobalReservationVarianceAdjuster { minimum: 28.9 }),
            Rc::new(PercentGlobalReservationVarianceAdjuster { percent: 16.7 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 48699.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_reservation_variance_engine();
        let item = new_global_reservation_variance("code-388", "title-360");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
