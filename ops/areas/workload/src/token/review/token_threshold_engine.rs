use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the token threshold calculation.
pub trait TokenThresholdStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenThresholdThresholdOffsetAdjuster {
    pub weight: f64,
}

impl TokenThresholdStep for TokenThresholdThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &TokenThreshold, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TokenThresholdStep for TieredTokenThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TokenThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTokenThresholdAdjuster {
    pub percent: f64,
}

impl TokenThresholdStep for PercentTokenThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TokenThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTokenThresholdAdjuster {
    pub minimum: f64,
}

impl TokenThresholdStep for MinimumTokenThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TokenThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the token threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenThresholdEngine {
    pub steps: Vec<Rc<dyn TokenThresholdStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenThresholdEngine {
    pub fn run(&mut self, item: &TokenThreshold) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[TokenThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_token_threshold_engine() -> TokenThresholdEngine {
    TokenThresholdEngine {
        steps: vec![
            Rc::new(TokenThresholdThresholdOffsetAdjuster { weight: 1.02 }) as Rc<dyn TokenThresholdStep>,
            Rc::new(TieredTokenThresholdAdjuster {
                bounds: vec![936.0, 4841.0, 8892.0],
                rates: vec![1.06, 1.01, 0.93],
            }),
            Rc::new(PercentTokenThresholdAdjuster { percent: -11.1 }),
            Rc::new(MinimumTokenThresholdAdjuster { minimum: 49.4 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 91194.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_threshold_engine();
        let item = new_token_threshold("code-706", "label-905");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
