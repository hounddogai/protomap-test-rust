use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the dns record forecast calculation.
pub trait DnsRecordForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordForecastAdjuster {
    pub minimum: f64,
}

impl DnsRecordForecastStep for MinimumDnsRecordForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordForecastQuantityOffsetAdjuster {
    pub weight: f64,
}

impl DnsRecordForecastStep for DnsRecordForecastQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &DnsRecordForecast, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDnsRecordForecastAdjuster {
    pub step: f64,
}

impl DnsRecordForecastStep for RoundingDnsRecordForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DnsRecordForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the dns record forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordForecastEngine {
    pub steps: Vec<Rc<dyn DnsRecordForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordForecastEngine {
    pub fn run(&mut self, item: &DnsRecordForecast) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DnsRecordForecastStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_dns_record_forecast_engine() -> DnsRecordForecastEngine {
    DnsRecordForecastEngine {
        steps: vec![
            Rc::new(MinimumDnsRecordForecastAdjuster { minimum: 29.5 }) as Rc<dyn DnsRecordForecastStep>,
            Rc::new(DnsRecordForecastQuantityOffsetAdjuster { weight: 0.39 }),
            Rc::new(RoundingDnsRecordForecastAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 71638.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_forecast_engine();
        let item = new_dns_record_forecast("code-276", "name-571");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
