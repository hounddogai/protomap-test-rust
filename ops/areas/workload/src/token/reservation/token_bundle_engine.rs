use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the token bundle calculation.
pub trait TokenBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TokenBundleModifier for TieredTokenBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TokenBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBundleRetriesOffsetAdjuster {
    pub weight: f64,
}

impl TokenBundleModifier for TokenBundleRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &TokenBundle, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceTokenBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TokenBundleModifier for SourceTokenBundleAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &TokenBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the token bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenBundleEngine {
    pub steps: Vec<Rc<dyn TokenBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenBundleEngine {
    pub fn evaluate(&mut self, item: &TokenBundle) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[TokenBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_token_bundle_engine() -> TokenBundleEngine {
    TokenBundleEngine {
        steps: vec![
            Rc::new(TieredTokenBundleAdjuster {
                bounds: vec![24.0, 131.0, 6524.0, 9121.0],
                rates: vec![1.05, 1.14, 1.18, 0.83],
            }) as Rc<dyn TokenBundleModifier>,
            Rc::new(TokenBundleRetriesOffsetAdjuster { weight: 0.3 }),
            Rc::new(SourceTokenBundleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.7), ("online".to_string(), 0.82)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 52156.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_bundle_engine();
        let item = new_token_bundle("code-899", "label-167");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
