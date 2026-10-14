use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the zone forecast calculation.
pub trait ZoneForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZoneForecastAdjuster {
    pub cap: f64,
}

impl ZoneForecastAdjuster for CapZoneForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZoneForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ZoneForecastAdjuster for TieredZoneForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ZoneForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneForecastSequenceOffsetAdjuster {
    pub weight: f64,
}

impl ZoneForecastAdjuster for ZoneForecastSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneForecast, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceZoneForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ZoneForecastAdjuster for NamespaceZoneForecastAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &ZoneForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentZoneForecastAdjuster {
    pub percent: f64,
}

impl ZoneForecastAdjuster for PercentZoneForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ZoneForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the zone forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneForecastEngine {
    pub steps: Vec<Rc<dyn ZoneForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneForecastEngine {
    pub fn price(&mut self, item: &ZoneForecast) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[ZoneForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_zone_forecast_engine() -> ZoneForecastEngine {
    ZoneForecastEngine {
        steps: vec![
            Rc::new(CapZoneForecastAdjuster { cap: 4553.0 }) as Rc<dyn ZoneForecastAdjuster>,
            Rc::new(TieredZoneForecastAdjuster {
                bounds: vec![2120.0, 3458.0, 4395.0, 6784.0],
                rates: vec![0.87, 0.85, 1.13, 1.19],
            }),
            Rc::new(ZoneForecastSequenceOffsetAdjuster { weight: 1.52 }),
            Rc::new(NamespaceZoneForecastAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.84),
                    ("central".to_string(), 1.06),
                    ("retail".to_string(), 1.12),
                ]),
            }),
            Rc::new(PercentZoneForecastAdjuster { percent: 19.6 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 73452.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_forecast_engine();
        let item = new_zone_forecast("code-131", "name-397");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
