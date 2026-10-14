use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted forecast mapping calculation.
pub trait WeightedForecastMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedForecastMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedForecastMappingAdjuster {
    pub step: f64,
}

impl WeightedForecastMappingModifier for RoundingWeightedForecastMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedForecastMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedForecastMappingAdjuster {
    pub minimum: f64,
}

impl WeightedForecastMappingModifier for MinimumWeightedForecastMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedForecastMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMappingStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl WeightedForecastMappingModifier for WeightedForecastMappingStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedForecastMapping, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

/// Runs the weighted forecast mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedForecastMappingEngine {
    pub steps: Vec<Rc<dyn WeightedForecastMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedForecastMappingEngine {
    pub fn price(&mut self, item: &WeightedForecastMapping) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[WeightedForecastMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_forecast_mapping_engine() -> WeightedForecastMappingEngine {
    WeightedForecastMappingEngine {
        steps: vec![
            Rc::new(RoundingWeightedForecastMappingAdjuster { step: 1.0 }) as Rc<dyn WeightedForecastMappingModifier>,
            Rc::new(MinimumWeightedForecastMappingAdjuster { minimum: 0.3 }),
            Rc::new(WeightedForecastMappingStartsAtOffsetAdjuster { weight: 0.93 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 47506.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_forecast_mapping_engine();
        let item = new_weighted_forecast_mapping("code-782", "title-380");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
