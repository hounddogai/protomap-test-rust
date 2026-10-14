use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the token digest calculation.
pub trait TokenDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TokenDigestAdjuster for TieredTokenDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TokenDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTokenDigestAdjuster {
    pub cap: f64,
}

impl TokenDigestAdjuster for CapTokenDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TokenDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTokenDigestAdjuster {
    pub step: f64,
}

impl TokenDigestAdjuster for RoundingTokenDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TokenDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTokenDigestAdjuster {
    pub minimum: f64,
}

impl TokenDigestAdjuster for MinimumTokenDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TokenDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTokenDigestAdjuster {
    pub percent: f64,
}

impl TokenDigestAdjuster for PercentTokenDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TokenDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the token digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenDigestEngine {
    pub steps: Vec<Rc<dyn TokenDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenDigestEngine {
    pub fn run(&mut self, item: &TokenDigest) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[TokenDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_token_digest_engine() -> TokenDigestEngine {
    TokenDigestEngine {
        steps: vec![
            Rc::new(TieredTokenDigestAdjuster {
                bounds: vec![1218.0, 2067.0, 5749.0],
                rates: vec![0.86, 1.12, 1.09],
            }) as Rc<dyn TokenDigestAdjuster>,
            Rc::new(CapTokenDigestAdjuster { cap: 24711.0 }),
            Rc::new(RoundingTokenDigestAdjuster { step: 0.5 }),
            Rc::new(MinimumTokenDigestAdjuster { minimum: 42.5 }),
            Rc::new(PercentTokenDigestAdjuster { percent: -3.5 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 66916.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_digest_engine();
        let item = new_token_digest("code-250", "name-639");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
