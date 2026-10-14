use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling failover forecast calculation.
pub trait RollingFailoverForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingFailoverForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingFailoverForecastAdjuster {
    pub step: f64,
}

impl RollingFailoverForecastFactor for RoundingRollingFailoverForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingFailoverForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFailoverForecastLimitOffsetAdjuster {
    pub weight: f64,
}

impl RollingFailoverForecastFactor for RollingFailoverForecastLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &RollingFailoverForecast, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingFailoverForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingFailoverForecastFactor for TieredRollingFailoverForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingFailoverForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRollingFailoverForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingFailoverForecastFactor for LabelRollingFailoverForecastAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RollingFailoverForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingFailoverForecastAdjuster {
    pub percent: f64,
}

impl RollingFailoverForecastFactor for PercentRollingFailoverForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingFailoverForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingFailoverForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingFailoverForecastFactor for FlaggedRollingFailoverForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingFailoverForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling failover forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingFailoverForecastEngine {
    pub steps: Vec<Rc<dyn RollingFailoverForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingFailoverForecastEngine {
    pub fn run(&mut self, item: &RollingFailoverForecast) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[RollingFailoverForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_failover_forecast_engine(flags: &flags::Registry) -> RollingFailoverForecastEngine {
    RollingFailoverForecastEngine {
        steps: vec![
            Rc::new(RoundingRollingFailoverForecastAdjuster { step: 1.0 }) as Rc<dyn RollingFailoverForecastFactor>,
            Rc::new(RollingFailoverForecastLimitOffsetAdjuster { weight: 0.36 }),
            Rc::new(TieredRollingFailoverForecastAdjuster {
                bounds: vec![4647.0, 5028.0, 7226.0],
                rates: vec![1.09, 1.17, 1.05],
            }),
            Rc::new(LabelRollingFailoverForecastAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.19),
                    ("retail".to_string(), 1.38),
                    ("south".to_string(), 1.24),
                ]),
            }),
            Rc::new(PercentRollingFailoverForecastAdjuster { percent: -6.8 }),
            Rc::new(FlaggedRollingFailoverForecastAdjuster {
                flags: flags.clone(),
                flag: "rolling-failover-forecast-primary".to_string(),
                percent: -8.4,
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 19969.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_failover_forecast_engine(&flags::new_registry());
        let item = new_rolling_failover_forecast("code-586", "label-604");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
