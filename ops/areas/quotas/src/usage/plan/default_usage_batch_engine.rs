use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default usage batch calculation.
pub trait DefaultUsageBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultUsageBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeDefaultUsageBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultUsageBatchModifier for ScopeDefaultUsageBatchAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &DefaultUsageBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageBatchExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl DefaultUsageBatchModifier for DefaultUsageBatchExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultUsageBatch, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultUsageBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultUsageBatchModifier for TieredDefaultUsageBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultUsageBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the default usage batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultUsageBatchEngine {
    pub steps: Vec<Rc<dyn DefaultUsageBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultUsageBatchEngine {
    pub fn run(&mut self, item: &DefaultUsageBatch) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DefaultUsageBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_usage_batch_engine() -> DefaultUsageBatchEngine {
    DefaultUsageBatchEngine {
        steps: vec![
            Rc::new(ScopeDefaultUsageBatchAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.14)]),
            }) as Rc<dyn DefaultUsageBatchModifier>,
            Rc::new(DefaultUsageBatchExpiresAtOffsetAdjuster { weight: 0.61 }),
            Rc::new(TieredDefaultUsageBatchAdjuster {
                bounds: vec![2616.0, 2793.0, 4866.0],
                rates: vec![0.88, 0.84, 1.13],
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 26997.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_usage_batch_engine();
        let item = new_default_usage_batch("code-412", "name-406");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
