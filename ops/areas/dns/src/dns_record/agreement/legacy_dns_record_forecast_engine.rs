use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the legacy dns record forecast calculation.
pub trait LegacyDnsRecordForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyDnsRecordForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyDnsRecordForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyDnsRecordForecastFactor for TieredLegacyDnsRecordForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyDnsRecordForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyDnsRecordForecastAdjuster {
    pub cap: f64,
}

impl LegacyDnsRecordForecastFactor for CapLegacyDnsRecordForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyDnsRecordForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyDnsRecordForecastAdjuster {
    pub minimum: f64,
}

impl LegacyDnsRecordForecastFactor for MinimumLegacyDnsRecordForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyDnsRecordForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDnsRecordForecastLimitOffsetAdjuster {
    pub weight: f64,
}

impl LegacyDnsRecordForecastFactor for LegacyDnsRecordForecastLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyDnsRecordForecast, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

/// Runs the legacy dns record forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyDnsRecordForecastEngine {
    pub steps: Vec<Rc<dyn LegacyDnsRecordForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyDnsRecordForecastEngine {
    pub fn evaluate(&mut self, item: &LegacyDnsRecordForecast) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyDnsRecordForecastFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyDnsRecordForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_dns_record_forecast_engine() -> LegacyDnsRecordForecastEngine {
    LegacyDnsRecordForecastEngine {
        steps: vec![
            Rc::new(TieredLegacyDnsRecordForecastAdjuster {
                bounds: vec![3164.0, 6707.0],
                rates: vec![0.95, 1.19],
            }) as Rc<dyn LegacyDnsRecordForecastFactor>,
            Rc::new(CapLegacyDnsRecordForecastAdjuster { cap: 17177.0 }),
            Rc::new(MinimumLegacyDnsRecordForecastAdjuster { minimum: 25.4 }),
            Rc::new(LegacyDnsRecordForecastLimitOffsetAdjuster { weight: 1.76 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 47849.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_dns_record_forecast_engine();
        let item = new_legacy_dns_record_forecast("code-747", "name-533");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
