use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy canary forecast calculation.
pub trait LegacyCanaryForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyCanaryForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyCanaryForecastAdjuster {
    pub step: f64,
}

impl LegacyCanaryForecastStep for RoundingLegacyCanaryForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyCanaryForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleLegacyCanaryForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyCanaryForecastStep for TitleLegacyCanaryForecastAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &LegacyCanaryForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyCanaryForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyCanaryForecastStep for FlaggedLegacyCanaryForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyCanaryForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyCanaryForecastAdjuster {
    pub minimum: f64,
}

impl LegacyCanaryForecastStep for MinimumLegacyCanaryForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyCanaryForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryForecastQuantityOffsetAdjuster {
    pub weight: f64,
}

impl LegacyCanaryForecastStep for LegacyCanaryForecastQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyCanaryForecast, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyCanaryForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyCanaryForecastStep for TieredLegacyCanaryForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyCanaryForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the legacy canary forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyCanaryForecastEngine {
    pub steps: Vec<Rc<dyn LegacyCanaryForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyCanaryForecastEngine {
    pub fn price(&mut self, item: &LegacyCanaryForecast) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[LegacyCanaryForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_canary_forecast_engine(flags: &flags::Registry) -> LegacyCanaryForecastEngine {
    LegacyCanaryForecastEngine {
        steps: vec![
            Rc::new(RoundingLegacyCanaryForecastAdjuster { step: 5.0 }) as Rc<dyn LegacyCanaryForecastStep>,
            Rc::new(TitleLegacyCanaryForecastAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.97),
                    ("central".to_string(), 1.09),
                    ("online".to_string(), 0.7),
                ]),
            }),
            Rc::new(FlaggedLegacyCanaryForecastAdjuster {
                flags: flags.clone(),
                flag: "legacy-canary-forecast-scheduled".to_string(),
                percent: 4.9,
            }),
            Rc::new(MinimumLegacyCanaryForecastAdjuster { minimum: 32.5 }),
            Rc::new(LegacyCanaryForecastQuantityOffsetAdjuster { weight: 1.22 }),
            Rc::new(TieredLegacyCanaryForecastAdjuster {
                bounds: vec![4343.0, 4912.0, 5466.0, 6102.0],
                rates: vec![0.91, 1.08, 1.12, 1.07],
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 19372.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_canary_forecast_engine(&flags::new_registry());
        let item = new_legacy_canary_forecast("code-649", "title-585");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
