use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback migration forecast calculation.
pub trait FallbackMigrationForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackMigrationForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackMigrationForecastAdjuster {
    pub percent: f64,
}

impl FallbackMigrationForecastFactor for PercentFallbackMigrationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackMigrationForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackMigrationForecastAdjuster {
    pub minimum: f64,
}

impl FallbackMigrationForecastFactor for MinimumFallbackMigrationForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackMigrationForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameFallbackMigrationForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackMigrationForecastFactor for NameFallbackMigrationForecastAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &FallbackMigrationForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackMigrationForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackMigrationForecastFactor for FlaggedFallbackMigrationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackMigrationForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackMigrationForecastAdjuster {
    pub cap: f64,
}

impl FallbackMigrationForecastFactor for CapFallbackMigrationForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackMigrationForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackMigrationForecastAdjuster {
    pub step: f64,
}

impl FallbackMigrationForecastFactor for RoundingFallbackMigrationForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackMigrationForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the fallback migration forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackMigrationForecastEngine {
    pub steps: Vec<Rc<dyn FallbackMigrationForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackMigrationForecastEngine {
    pub fn compute(&mut self, item: &FallbackMigrationForecast) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackMigrationForecastFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackMigrationForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_migration_forecast_engine(flags: &flags::Registry) -> FallbackMigrationForecastEngine {
    FallbackMigrationForecastEngine {
        steps: vec![
            Rc::new(PercentFallbackMigrationForecastAdjuster { percent: 12.3 })
                as Rc<dyn FallbackMigrationForecastFactor>,
            Rc::new(MinimumFallbackMigrationForecastAdjuster { minimum: 44.4 }),
            Rc::new(NameFallbackMigrationForecastAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.94), ("central".to_string(), 1.26)]),
            }),
            Rc::new(FlaggedFallbackMigrationForecastAdjuster {
                flags: flags.clone(),
                flag: "fallback-migration-forecast-legacy".to_string(),
                percent: -4.1,
            }),
            Rc::new(CapFallbackMigrationForecastAdjuster { cap: 48329.0 }),
            Rc::new(RoundingFallbackMigrationForecastAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 46142.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_migration_forecast_engine(&flags::new_registry());
        let item = new_fallback_migration_forecast("code-160", "name-650");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
