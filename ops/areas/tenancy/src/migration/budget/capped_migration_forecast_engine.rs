use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped migration forecast calculation.
pub trait CappedMigrationForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedMigrationForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedMigrationForecastAdjuster {
    pub minimum: f64,
}

impl CappedMigrationForecastAdjuster for MinimumCappedMigrationForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedMigrationForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedMigrationForecastAdjuster {
    pub step: f64,
}

impl CappedMigrationForecastAdjuster for RoundingCappedMigrationForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedMigrationForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedMigrationForecastAdjuster {
    pub percent: f64,
}

impl CappedMigrationForecastAdjuster for PercentCappedMigrationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedMigrationForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedMigrationForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedMigrationForecastAdjuster for TieredCappedMigrationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedMigrationForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedMigrationForecastAdjuster {
    pub cap: f64,
}

impl CappedMigrationForecastAdjuster for CapCappedMigrationForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedMigrationForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationForecastTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl CappedMigrationForecastAdjuster for CappedMigrationForecastTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &CappedMigrationForecast, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

/// Runs the capped migration forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedMigrationForecastEngine {
    pub steps: Vec<Rc<dyn CappedMigrationForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedMigrationForecastEngine {
    pub fn run(&mut self, item: &CappedMigrationForecast) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[CappedMigrationForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_migration_forecast_engine() -> CappedMigrationForecastEngine {
    CappedMigrationForecastEngine {
        steps: vec![
            Rc::new(MinimumCappedMigrationForecastAdjuster { minimum: 48.1 })
                as Rc<dyn CappedMigrationForecastAdjuster>,
            Rc::new(RoundingCappedMigrationForecastAdjuster { step: 1.0 }),
            Rc::new(PercentCappedMigrationForecastAdjuster { percent: 22.7 }),
            Rc::new(TieredCappedMigrationForecastAdjuster {
                bounds: vec![2656.0, 5261.0, 5917.0],
                rates: vec![0.88, 0.93, 0.89],
            }),
            Rc::new(CapCappedMigrationForecastAdjuster { cap: 39496.0 }),
            Rc::new(CappedMigrationForecastTtlSecondsOffsetAdjuster { weight: 1.09 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 9906.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_migration_forecast_engine();
        let item = new_capped_migration_forecast("code-140", "title-913");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
