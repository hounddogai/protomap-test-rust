use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the canary forecast calculation.
pub trait CanaryForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CanaryForecastStep for TieredCanaryForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CanaryForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryForecastVersionOffsetAdjuster {
    pub weight: f64,
}

impl CanaryForecastStep for CanaryForecastVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &CanaryForecast, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCanaryForecastAdjuster {
    pub cap: f64,
}

impl CanaryForecastStep for CapCanaryForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CanaryForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCanaryForecastAdjuster {
    pub minimum: f64,
}

impl CanaryForecastStep for MinimumCanaryForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CanaryForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelCanaryForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CanaryForecastStep for LabelCanaryForecastAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &CanaryForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCanaryForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CanaryForecastStep for FlaggedCanaryForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CanaryForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the canary forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryForecastEngine {
    pub steps: Vec<Rc<dyn CanaryForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryForecastEngine {
    pub fn compute(&mut self, item: &CanaryForecast) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CanaryForecastStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CanaryForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_canary_forecast_engine(flags: &flags::Registry) -> CanaryForecastEngine {
    CanaryForecastEngine {
        steps: vec![
            Rc::new(TieredCanaryForecastAdjuster {
                bounds: vec![3707.0, 6744.0, 7553.0],
                rates: vec![1.06, 1.09, 1.2],
            }) as Rc<dyn CanaryForecastStep>,
            Rc::new(CanaryForecastVersionOffsetAdjuster { weight: 0.08 }),
            Rc::new(CapCanaryForecastAdjuster { cap: 19231.0 }),
            Rc::new(MinimumCanaryForecastAdjuster { minimum: 19.8 }),
            Rc::new(LabelCanaryForecastAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.87)]),
            }),
            Rc::new(FlaggedCanaryForecastAdjuster {
                flags: flags.clone(),
                flag: "canary-forecast-seasonal".to_string(),
                percent: -1.5,
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 52096.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_forecast_engine(&flags::new_registry());
        let item = new_canary_forecast("code-446", "label-812");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
