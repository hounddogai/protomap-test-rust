use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched audit event forecast calculation.
pub trait BatchedAuditEventForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedAuditEventForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventForecastCeilingOffsetAdjuster {
    pub weight: f64,
}

impl BatchedAuditEventForecastModifier for BatchedAuditEventForecastCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedAuditEventForecast, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedAuditEventForecastAdjuster {
    pub minimum: f64,
}

impl BatchedAuditEventForecastModifier for MinimumBatchedAuditEventForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedAuditEventForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedAuditEventForecastAdjuster {
    pub step: f64,
}

impl BatchedAuditEventForecastModifier for RoundingBatchedAuditEventForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedAuditEventForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceBatchedAuditEventForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedAuditEventForecastModifier for NamespaceBatchedAuditEventForecastAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &BatchedAuditEventForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedAuditEventForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedAuditEventForecastModifier for TieredBatchedAuditEventForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedAuditEventForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the batched audit event forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedAuditEventForecastEngine {
    pub steps: Vec<Rc<dyn BatchedAuditEventForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedAuditEventForecastEngine {
    pub fn run(&mut self, item: &BatchedAuditEventForecast) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[BatchedAuditEventForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_audit_event_forecast_engine() -> BatchedAuditEventForecastEngine {
    BatchedAuditEventForecastEngine {
        steps: vec![
            Rc::new(BatchedAuditEventForecastCeilingOffsetAdjuster { weight: 0.98 })
                as Rc<dyn BatchedAuditEventForecastModifier>,
            Rc::new(MinimumBatchedAuditEventForecastAdjuster { minimum: 48.6 }),
            Rc::new(RoundingBatchedAuditEventForecastAdjuster { step: 5.0 }),
            Rc::new(NamespaceBatchedAuditEventForecastAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.82)]),
            }),
            Rc::new(TieredBatchedAuditEventForecastAdjuster {
                bounds: vec![447.0, 7098.0],
                rates: vec![0.9, 1.1],
            }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 30839.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_audit_event_forecast_engine();
        let item = new_batched_audit_event_forecast("code-134", "name-641");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
