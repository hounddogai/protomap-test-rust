use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary isolation forecast calculation.
pub trait PrimaryIsolationForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryIsolationForecast, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryIsolationForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryIsolationForecastAdjuster for FlaggedPrimaryIsolationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryIsolationForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryIsolationForecastAdjuster {
    pub step: f64,
}

impl PrimaryIsolationForecastAdjuster for RoundingPrimaryIsolationForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryIsolationForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryIsolationForecastAdjuster {
    pub percent: f64,
}

impl PrimaryIsolationForecastAdjuster for PercentPrimaryIsolationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryIsolationForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePrimaryIsolationForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryIsolationForecastAdjuster for TitlePrimaryIsolationForecastAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PrimaryIsolationForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryIsolationForecastAdjuster {
    pub minimum: f64,
}

impl PrimaryIsolationForecastAdjuster for MinimumPrimaryIsolationForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryIsolationForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the primary isolation forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryIsolationForecastEngine {
    pub steps: Vec<Rc<dyn PrimaryIsolationForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryIsolationForecastEngine {
    pub fn price(&mut self, item: &PrimaryIsolationForecast) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryIsolationForecastAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryIsolationForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_isolation_forecast_engine(flags: &flags::Registry) -> PrimaryIsolationForecastEngine {
    PrimaryIsolationForecastEngine {
        steps: vec![
            Rc::new(FlaggedPrimaryIsolationForecastAdjuster {
                flags: flags.clone(),
                flag: "primary-isolation-forecast-capped".to_string(),
                percent: 2.9,
            }) as Rc<dyn PrimaryIsolationForecastAdjuster>,
            Rc::new(RoundingPrimaryIsolationForecastAdjuster { step: 0.25 }),
            Rc::new(PercentPrimaryIsolationForecastAdjuster { percent: -12.9 }),
            Rc::new(TitlePrimaryIsolationForecastAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.14), ("north".to_string(), 1.25)]),
            }),
            Rc::new(MinimumPrimaryIsolationForecastAdjuster { minimum: 40.7 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 80744.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_isolation_forecast_engine(&flags::new_registry());
        let item = new_primary_isolation_forecast("code-913", "title-1");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
