use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted alert quota calculation.
pub trait WeightedAlertQuotaStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedAlertQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedAlertQuotaAdjuster {
    pub minimum: f64,
}

impl WeightedAlertQuotaStep for MinimumWeightedAlertQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedAlertQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedAlertQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedAlertQuotaStep for TieredWeightedAlertQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedAlertQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertQuotaAmountOffsetAdjuster {
    pub weight: f64,
}

impl WeightedAlertQuotaStep for WeightedAlertQuotaAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedAlertQuota, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedAlertQuotaAdjuster {
    pub percent: f64,
}

impl WeightedAlertQuotaStep for PercentWeightedAlertQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedAlertQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted alert quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedAlertQuotaEngine {
    pub steps: Vec<Rc<dyn WeightedAlertQuotaStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedAlertQuotaEngine {
    pub fn evaluate(&mut self, item: &WeightedAlertQuota) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[WeightedAlertQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_alert_quota_engine() -> WeightedAlertQuotaEngine {
    WeightedAlertQuotaEngine {
        steps: vec![
            Rc::new(MinimumWeightedAlertQuotaAdjuster { minimum: 13.9 }) as Rc<dyn WeightedAlertQuotaStep>,
            Rc::new(TieredWeightedAlertQuotaAdjuster {
                bounds: vec![1241.0, 4940.0, 5083.0],
                rates: vec![1.17, 1.03, 0.91],
            }),
            Rc::new(WeightedAlertQuotaAmountOffsetAdjuster { weight: 0.34 }),
            Rc::new(PercentWeightedAlertQuotaAdjuster { percent: -23.0 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 95469.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_alert_quota_engine();
        let item = new_weighted_alert_quota("code-747", "label-966");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
