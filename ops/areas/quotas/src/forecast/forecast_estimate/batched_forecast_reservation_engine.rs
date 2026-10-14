use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched forecast reservation calculation.
pub trait BatchedForecastReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedForecastReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastReservationQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl BatchedForecastReservationFactor for BatchedForecastReservationQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedForecastReservation, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedForecastReservationAdjuster {
    pub minimum: f64,
}

impl BatchedForecastReservationFactor for MinimumBatchedForecastReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedForecastReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedForecastReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedForecastReservationFactor for FlaggedBatchedForecastReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedForecastReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedForecastReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedForecastReservationFactor for TieredBatchedForecastReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedForecastReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedForecastReservationAdjuster {
    pub percent: f64,
}

impl BatchedForecastReservationFactor for PercentBatchedForecastReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedForecastReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedForecastReservationAdjuster {
    pub cap: f64,
}

impl BatchedForecastReservationFactor for CapBatchedForecastReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedForecastReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the batched forecast reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedForecastReservationEngine {
    pub steps: Vec<Rc<dyn BatchedForecastReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedForecastReservationEngine {
    pub fn compute(&mut self, item: &BatchedForecastReservation) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[BatchedForecastReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_forecast_reservation_engine(flags: &flags::Registry) -> BatchedForecastReservationEngine {
    BatchedForecastReservationEngine {
        steps: vec![
            Rc::new(BatchedForecastReservationQuotaUnitsOffsetAdjuster { weight: 1.17 })
                as Rc<dyn BatchedForecastReservationFactor>,
            Rc::new(MinimumBatchedForecastReservationAdjuster { minimum: 36.8 }),
            Rc::new(FlaggedBatchedForecastReservationAdjuster {
                flags: flags.clone(),
                flag: "batched-forecast-reservation-weighted".to_string(),
                percent: -1.4,
            }),
            Rc::new(TieredBatchedForecastReservationAdjuster {
                bounds: vec![1368.0, 2343.0, 7427.0],
                rates: vec![0.85, 1.16, 1.05],
            }),
            Rc::new(PercentBatchedForecastReservationAdjuster { percent: -12.0 }),
            Rc::new(CapBatchedForecastReservationAdjuster { cap: 28811.0 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 61828.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_forecast_reservation_engine(&flags::new_registry());
        let item = new_batched_forecast_reservation("code-774", "label-786");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
