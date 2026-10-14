use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted forecast reservation calculation.
pub trait WeightedForecastReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedForecastReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelWeightedForecastReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedForecastReservationFactor for LabelWeightedForecastReservationAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &WeightedForecastReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedForecastReservationAdjuster {
    pub step: f64,
}

impl WeightedForecastReservationFactor for RoundingWeightedForecastReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedForecastReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedForecastReservationAdjuster {
    pub cap: f64,
}

impl WeightedForecastReservationFactor for CapWeightedForecastReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedForecastReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedForecastReservationAdjuster {
    pub minimum: f64,
}

impl WeightedForecastReservationFactor for MinimumWeightedForecastReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedForecastReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedForecastReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedForecastReservationFactor for FlaggedWeightedForecastReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedForecastReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedForecastReservationAdjuster {
    pub percent: f64,
}

impl WeightedForecastReservationFactor for PercentWeightedForecastReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedForecastReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted forecast reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedForecastReservationEngine {
    pub steps: Vec<Rc<dyn WeightedForecastReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedForecastReservationEngine {
    pub fn evaluate(&mut self, item: &WeightedForecastReservation) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedForecastReservationFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedForecastReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_forecast_reservation_engine(flags: &flags::Registry) -> WeightedForecastReservationEngine {
    WeightedForecastReservationEngine {
        steps: vec![
            Rc::new(LabelWeightedForecastReservationAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.84), ("west".to_string(), 0.93)]),
            }) as Rc<dyn WeightedForecastReservationFactor>,
            Rc::new(RoundingWeightedForecastReservationAdjuster { step: 1.0 }),
            Rc::new(CapWeightedForecastReservationAdjuster { cap: 49277.0 }),
            Rc::new(MinimumWeightedForecastReservationAdjuster { minimum: 22.8 }),
            Rc::new(FlaggedWeightedForecastReservationAdjuster {
                flags: flags.clone(),
                flag: "weighted-forecast-reservation-lenient".to_string(),
                percent: 0.9,
            }),
            Rc::new(PercentWeightedForecastReservationAdjuster { percent: 16.9 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 40016.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_forecast_reservation_engine(&flags::new_registry());
        let item = new_weighted_forecast_reservation("code-902", "label-711");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
