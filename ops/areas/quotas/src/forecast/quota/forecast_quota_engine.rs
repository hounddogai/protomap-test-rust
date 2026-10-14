use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the forecast quota calculation.
pub trait ForecastQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ForecastQuotaModifier for TieredForecastQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ForecastQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelForecastQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ForecastQuotaModifier for ChannelForecastQuotaAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &ForecastQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastQuotaAdjuster {
    pub minimum: f64,
}

impl ForecastQuotaModifier for MinimumForecastQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the forecast quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastQuotaEngine {
    pub steps: Vec<Rc<dyn ForecastQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastQuotaEngine {
    pub fn price(&mut self, item: &ForecastQuota) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ForecastQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_forecast_quota_engine() -> ForecastQuotaEngine {
    ForecastQuotaEngine {
        steps: vec![
            Rc::new(TieredForecastQuotaAdjuster {
                bounds: vec![1438.0, 5749.0],
                rates: vec![0.82, 1.15],
            }) as Rc<dyn ForecastQuotaModifier>,
            Rc::new(ChannelForecastQuotaAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.8),
                    ("east".to_string(), 0.84),
                    ("north".to_string(), 0.87),
                ]),
            }),
            Rc::new(MinimumForecastQuotaAdjuster { minimum: 31.3 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 56527.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_quota_engine();
        let item = new_forecast_quota("code-887", "title-345");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
