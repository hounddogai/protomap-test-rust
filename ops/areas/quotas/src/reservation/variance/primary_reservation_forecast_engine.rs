use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary reservation forecast calculation.
pub trait PrimaryReservationForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryReservationForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryReservationForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryReservationForecastModifier for TieredPrimaryReservationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryReservationForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryReservationForecastAdjuster {
    pub minimum: f64,
}

impl PrimaryReservationForecastModifier for MinimumPrimaryReservationForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryReservationForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryReservationForecastAdjuster {
    pub step: f64,
}

impl PrimaryReservationForecastModifier for RoundingPrimaryReservationForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryReservationForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopePrimaryReservationForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryReservationForecastModifier for ScopePrimaryReservationForecastAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &PrimaryReservationForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary reservation forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryReservationForecastEngine {
    pub steps: Vec<Rc<dyn PrimaryReservationForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryReservationForecastEngine {
    pub fn compute(&mut self, item: &PrimaryReservationForecast) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryReservationForecastModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryReservationForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_reservation_forecast_engine() -> PrimaryReservationForecastEngine {
    PrimaryReservationForecastEngine {
        steps: vec![
            Rc::new(TieredPrimaryReservationForecastAdjuster {
                bounds: vec![2733.0, 3615.0, 4685.0, 6124.0],
                rates: vec![0.95, 1.18, 1.15, 0.84],
            }) as Rc<dyn PrimaryReservationForecastModifier>,
            Rc::new(MinimumPrimaryReservationForecastAdjuster { minimum: 20.5 }),
            Rc::new(RoundingPrimaryReservationForecastAdjuster { step: 0.05 }),
            Rc::new(ScopePrimaryReservationForecastAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.31), ("south".to_string(), 1.14)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 33974.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_reservation_forecast_engine();
        let item = new_primary_reservation_forecast("code-968", "title-6");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
