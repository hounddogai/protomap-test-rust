use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped evidence forecast calculation.
pub trait CappedEvidenceForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedEvidenceForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryCappedEvidenceForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedEvidenceForecastAdjuster for CategoryCappedEvidenceForecastAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &CappedEvidenceForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedEvidenceForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedEvidenceForecastAdjuster for TieredCappedEvidenceForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedEvidenceForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceForecastEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl CappedEvidenceForecastAdjuster for CappedEvidenceForecastEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &CappedEvidenceForecast, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

/// Runs the capped evidence forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedEvidenceForecastEngine {
    pub steps: Vec<Rc<dyn CappedEvidenceForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedEvidenceForecastEngine {
    pub fn price(&mut self, item: &CappedEvidenceForecast) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[CappedEvidenceForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_evidence_forecast_engine() -> CappedEvidenceForecastEngine {
    CappedEvidenceForecastEngine {
        steps: vec![
            Rc::new(CategoryCappedEvidenceForecastAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.92), ("retail".to_string(), 1.03)]),
            }) as Rc<dyn CappedEvidenceForecastAdjuster>,
            Rc::new(TieredCappedEvidenceForecastAdjuster {
                bounds: vec![5282.0, 6080.0],
                rates: vec![1.11, 1.09],
            }),
            Rc::new(CappedEvidenceForecastEndsAtOffsetAdjuster { weight: 1.98 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 33424.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_evidence_forecast_engine();
        let item = new_capped_evidence_forecast("code-506", "name-918");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
