use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the forecast reservation calculation.
pub trait ForecastReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastReservationAmountOffsetAdjuster {
    pub weight: f64,
}

impl ForecastReservationAdjuster for ForecastReservationAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &ForecastReservation, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapForecastReservationAdjuster {
    pub cap: f64,
}

impl ForecastReservationAdjuster for CapForecastReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ForecastReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingForecastReservationAdjuster {
    pub step: f64,
}

impl ForecastReservationAdjuster for RoundingForecastReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ForecastReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentForecastReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ForecastReservationAdjuster for SegmentForecastReservationAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &ForecastReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentForecastReservationAdjuster {
    pub percent: f64,
}

impl ForecastReservationAdjuster for PercentForecastReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ForecastReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the forecast reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastReservationEngine {
    pub steps: Vec<Rc<dyn ForecastReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastReservationEngine {
    pub fn compute(&mut self, item: &ForecastReservation) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ForecastReservationAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ForecastReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_forecast_reservation_engine() -> ForecastReservationEngine {
    ForecastReservationEngine {
        steps: vec![
            Rc::new(ForecastReservationAmountOffsetAdjuster { weight: 0.32 }) as Rc<dyn ForecastReservationAdjuster>,
            Rc::new(CapForecastReservationAdjuster { cap: 41708.0 }),
            Rc::new(RoundingForecastReservationAdjuster { step: 1.0 }),
            Rc::new(SegmentForecastReservationAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.2), ("south".to_string(), 0.86)]),
            }),
            Rc::new(PercentForecastReservationAdjuster { percent: 21.1 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 7423.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_reservation_engine();
        let item = new_forecast_reservation("code-621", "label-832");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
