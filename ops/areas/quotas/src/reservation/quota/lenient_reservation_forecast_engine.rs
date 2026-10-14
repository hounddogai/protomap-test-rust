use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient reservation forecast calculation.
pub trait LenientReservationForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientReservationForecast, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientReservationForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientReservationForecastAdjuster for FlaggedLenientReservationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientReservationForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientReservationForecastAdjuster {
    pub minimum: f64,
}

impl LenientReservationForecastAdjuster for MinimumLenientReservationForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientReservationForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientReservationForecastAdjuster {
    pub percent: f64,
}

impl LenientReservationForecastAdjuster for PercentLenientReservationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientReservationForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationForecastRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl LenientReservationForecastAdjuster for LenientReservationForecastRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &LenientReservationForecast, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryLenientReservationForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientReservationForecastAdjuster for CategoryLenientReservationForecastAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &LenientReservationForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the lenient reservation forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientReservationForecastEngine {
    pub steps: Vec<Rc<dyn LenientReservationForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientReservationForecastEngine {
    pub fn evaluate(&mut self, item: &LenientReservationForecast) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[LenientReservationForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_reservation_forecast_engine(flags: &flags::Registry) -> LenientReservationForecastEngine {
    LenientReservationForecastEngine {
        steps: vec![
            Rc::new(FlaggedLenientReservationForecastAdjuster {
                flags: flags.clone(),
                flag: "lenient-reservation-forecast-seasonal".to_string(),
                percent: -0.6,
            }) as Rc<dyn LenientReservationForecastAdjuster>,
            Rc::new(MinimumLenientReservationForecastAdjuster { minimum: 10.0 }),
            Rc::new(PercentLenientReservationForecastAdjuster { percent: -21.6 }),
            Rc::new(LenientReservationForecastRotationDaysOffsetAdjuster { weight: 0.79 }),
            Rc::new(CategoryLenientReservationForecastAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.88),
                    ("north".to_string(), 1.29),
                    ("east".to_string(), 0.85),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 64745.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_reservation_forecast_engine(&flags::new_registry());
        let item = new_lenient_reservation_forecast("code-937", "name-644");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
