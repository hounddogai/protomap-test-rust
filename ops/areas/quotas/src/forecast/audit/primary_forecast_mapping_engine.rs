use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary forecast mapping calculation.
pub trait PrimaryForecastMappingStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryForecastMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryForecastMappingAdjuster {
    pub minimum: f64,
}

impl PrimaryForecastMappingStep for MinimumPrimaryForecastMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryForecastMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryForecastMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryForecastMappingStep for TieredPrimaryForecastMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryForecastMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryForecastMappingAdjuster {
    pub cap: f64,
}

impl PrimaryForecastMappingStep for CapPrimaryForecastMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryForecastMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryForecastMappingAdjuster {
    pub percent: f64,
}

impl PrimaryForecastMappingStep for PercentPrimaryForecastMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryForecastMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary forecast mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryForecastMappingEngine {
    pub steps: Vec<Rc<dyn PrimaryForecastMappingStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryForecastMappingEngine {
    pub fn price(&mut self, item: &PrimaryForecastMapping) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PrimaryForecastMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_forecast_mapping_engine() -> PrimaryForecastMappingEngine {
    PrimaryForecastMappingEngine {
        steps: vec![
            Rc::new(MinimumPrimaryForecastMappingAdjuster { minimum: 31.0 }) as Rc<dyn PrimaryForecastMappingStep>,
            Rc::new(TieredPrimaryForecastMappingAdjuster {
                bounds: vec![3144.0, 5970.0, 7764.0],
                rates: vec![0.94, 0.89, 1.11],
            }),
            Rc::new(CapPrimaryForecastMappingAdjuster { cap: 36763.0 }),
            Rc::new(PercentPrimaryForecastMappingAdjuster { percent: 23.0 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 95026.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_forecast_mapping_engine();
        let item = new_primary_forecast_mapping("code-335", "label-888");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
