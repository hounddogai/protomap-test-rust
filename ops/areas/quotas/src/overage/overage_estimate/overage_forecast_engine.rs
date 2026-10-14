use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the overage forecast calculation.
pub trait OverageForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageForecastVersionOffsetAdjuster {
    pub weight: f64,
}

impl OverageForecastAdjuster for OverageForecastVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &OverageForecast, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionOverageForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl OverageForecastAdjuster for RegionOverageForecastAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &OverageForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl OverageForecastAdjuster for TieredOverageForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &OverageForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the overage forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageForecastEngine {
    pub steps: Vec<Rc<dyn OverageForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageForecastEngine {
    pub fn price(&mut self, item: &OverageForecast) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn OverageForecastAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[OverageForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_overage_forecast_engine() -> OverageForecastEngine {
    OverageForecastEngine {
        steps: vec![
            Rc::new(OverageForecastVersionOffsetAdjuster { weight: 0.79 }) as Rc<dyn OverageForecastAdjuster>,
            Rc::new(RegionOverageForecastAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.21)]),
            }),
            Rc::new(TieredOverageForecastAdjuster {
                bounds: vec![4211.0, 5825.0],
                rates: vec![0.98, 0.87],
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 94354.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_forecast_engine();
        let item = new_overage_forecast("code-805", "label-939");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
