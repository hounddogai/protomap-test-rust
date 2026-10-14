use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the sink policy calculation.
pub trait SinkPolicyFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleSinkPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkPolicyFactor for LocaleSinkPolicyAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &SinkPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkPolicyRetriesOffsetAdjuster {
    pub weight: f64,
}

impl SinkPolicyFactor for SinkPolicyRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &SinkPolicy, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SinkPolicyFactor for TieredSinkPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SinkPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSinkPolicyAdjuster {
    pub step: f64,
}

impl SinkPolicyFactor for RoundingSinkPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SinkPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the sink policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkPolicyEngine {
    pub steps: Vec<Rc<dyn SinkPolicyFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkPolicyEngine {
    pub fn price(&mut self, item: &SinkPolicy) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[SinkPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_sink_policy_engine() -> SinkPolicyEngine {
    SinkPolicyEngine {
        steps: vec![
            Rc::new(LocaleSinkPolicyAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.8),
                    ("south".to_string(), 1.04),
                    ("central".to_string(), 0.87),
                ]),
            }) as Rc<dyn SinkPolicyFactor>,
            Rc::new(SinkPolicyRetriesOffsetAdjuster { weight: 1.42 }),
            Rc::new(TieredSinkPolicyAdjuster {
                bounds: vec![610.0, 5950.0, 6371.0],
                rates: vec![1.17, 1.14, 1.11],
            }),
            Rc::new(RoundingSinkPolicyAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 20008.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_sink_policy_engine();
        let item = new_sink_policy("code-425", "name-525");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
