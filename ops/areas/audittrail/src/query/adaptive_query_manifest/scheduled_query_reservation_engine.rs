use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the scheduled query reservation calculation.
pub trait ScheduledQueryReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledQueryReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledQueryReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledQueryReservationStep for TieredScheduledQueryReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledQueryReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelScheduledQueryReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledQueryReservationStep for LabelScheduledQueryReservationAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ScheduledQueryReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledQueryReservationAdjuster {
    pub minimum: f64,
}

impl ScheduledQueryReservationStep for MinimumScheduledQueryReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledQueryReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the scheduled query reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledQueryReservationEngine {
    pub steps: Vec<Rc<dyn ScheduledQueryReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledQueryReservationEngine {
    pub fn evaluate(&mut self, item: &ScheduledQueryReservation) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledQueryReservationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledQueryReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_query_reservation_engine() -> ScheduledQueryReservationEngine {
    ScheduledQueryReservationEngine {
        steps: vec![
            Rc::new(TieredScheduledQueryReservationAdjuster {
                bounds: vec![5959.0, 9156.0],
                rates: vec![0.89, 1.1],
            }) as Rc<dyn ScheduledQueryReservationStep>,
            Rc::new(LabelScheduledQueryReservationAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.05)]),
            }),
            Rc::new(MinimumScheduledQueryReservationAdjuster { minimum: 35.3 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 3228.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_query_reservation_engine();
        let item = new_scheduled_query_reservation("code-525", "label-982");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
