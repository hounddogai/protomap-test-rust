use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the rollout reservation calculation.
pub trait RolloutReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRolloutReservationAdjuster {
    pub cap: f64,
}

impl RolloutReservationAdjuster for CapRolloutReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RolloutReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRolloutReservationAdjuster {
    pub minimum: f64,
}

impl RolloutReservationAdjuster for MinimumRolloutReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RolloutReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRolloutReservationAdjuster {
    pub percent: f64,
}

impl RolloutReservationAdjuster for PercentRolloutReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RolloutReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RolloutReservationAdjuster for TieredRolloutReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RolloutReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleRolloutReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutReservationAdjuster for LocaleRolloutReservationAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &RolloutReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rollout reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutReservationEngine {
    pub steps: Vec<Rc<dyn RolloutReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutReservationEngine {
    pub fn run(&mut self, item: &RolloutReservation) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RolloutReservationAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RolloutReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rollout_reservation_engine() -> RolloutReservationEngine {
    RolloutReservationEngine {
        steps: vec![
            Rc::new(CapRolloutReservationAdjuster { cap: 3079.0 }) as Rc<dyn RolloutReservationAdjuster>,
            Rc::new(MinimumRolloutReservationAdjuster { minimum: 5.8 }),
            Rc::new(PercentRolloutReservationAdjuster { percent: 6.7 }),
            Rc::new(TieredRolloutReservationAdjuster {
                bounds: vec![1857.0, 5215.0, 5591.0, 7303.0],
                rates: vec![0.84, 1.03, 1.06, 1.06],
            }),
            Rc::new(LocaleRolloutReservationAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.82)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 13810.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_reservation_engine();
        let item = new_rollout_reservation("code-84", "label-831");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
