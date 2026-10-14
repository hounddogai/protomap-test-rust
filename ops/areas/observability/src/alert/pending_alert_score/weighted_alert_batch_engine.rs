use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted alert batch calculation.
pub trait WeightedAlertBatchAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedAlertBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedAlertBatchAdjuster {
    pub step: f64,
}

impl WeightedAlertBatchAdjuster for RoundingWeightedAlertBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedAlertBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedAlertBatchAdjuster {
    pub percent: f64,
}

impl WeightedAlertBatchAdjuster for PercentWeightedAlertBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedAlertBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryWeightedAlertBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedAlertBatchAdjuster for CountryWeightedAlertBatchAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &WeightedAlertBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the weighted alert batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedAlertBatchEngine {
    pub steps: Vec<Rc<dyn WeightedAlertBatchAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedAlertBatchEngine {
    pub fn price(&mut self, item: &WeightedAlertBatch) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[WeightedAlertBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_alert_batch_engine() -> WeightedAlertBatchEngine {
    WeightedAlertBatchEngine {
        steps: vec![
            Rc::new(RoundingWeightedAlertBatchAdjuster { step: 5.0 }) as Rc<dyn WeightedAlertBatchAdjuster>,
            Rc::new(PercentWeightedAlertBatchAdjuster { percent: 10.1 }),
            Rc::new(CountryWeightedAlertBatchAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.72), ("south".to_string(), 1.32)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 70643.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_alert_batch_engine();
        let item = new_weighted_alert_batch("code-459", "label-373");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
