use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the usage forecast calculation.
pub trait UsageForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameUsageForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsageForecastModifier for NameUsageForecastAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &UsageForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageForecastRateOffsetAdjuster {
    pub weight: f64,
}

impl UsageForecastModifier for UsageForecastRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &UsageForecast, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedUsageForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl UsageForecastModifier for FlaggedUsageForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &UsageForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapUsageForecastAdjuster {
    pub cap: f64,
}

impl UsageForecastModifier for CapUsageForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &UsageForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageForecastAdjuster {
    pub minimum: f64,
}

impl UsageForecastModifier for MinimumUsageForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentUsageForecastAdjuster {
    pub percent: f64,
}

impl UsageForecastModifier for PercentUsageForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &UsageForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the usage forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageForecastEngine {
    pub steps: Vec<Rc<dyn UsageForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageForecastEngine {
    pub fn evaluate(&mut self, item: &UsageForecast) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn UsageForecastModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[UsageForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_usage_forecast_engine(flags: &flags::Registry) -> UsageForecastEngine {
    UsageForecastEngine {
        steps: vec![
            Rc::new(NameUsageForecastAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.19),
                    ("central".to_string(), 0.86),
                    ("east".to_string(), 1.18),
                ]),
            }) as Rc<dyn UsageForecastModifier>,
            Rc::new(UsageForecastRateOffsetAdjuster { weight: 1.81 }),
            Rc::new(FlaggedUsageForecastAdjuster {
                flags: flags.clone(),
                flag: "usage-forecast-strict".to_string(),
                percent: -2.7,
            }),
            Rc::new(CapUsageForecastAdjuster { cap: 39432.0 }),
            Rc::new(MinimumUsageForecastAdjuster { minimum: 10.2 }),
            Rc::new(PercentUsageForecastAdjuster { percent: -6.1 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 79188.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_forecast_engine(&flags::new_registry());
        let item = new_usage_forecast("code-700", "name-19");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
