use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the nonce score calculation.
pub trait NonceScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentNonceScoreAdjuster {
    pub percent: f64,
}

impl NonceScoreAdjuster for PercentNonceScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &NonceScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceScoreAdjuster for TieredNonceScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceScoreGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl NonceScoreAdjuster for NonceScoreGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &NonceScore, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingNonceScoreAdjuster {
    pub step: f64,
}

impl NonceScoreAdjuster for RoundingNonceScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &NonceScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the nonce score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceScoreEngine {
    pub steps: Vec<Rc<dyn NonceScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceScoreEngine {
    pub fn price(&mut self, item: &NonceScore) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn NonceScoreAdjuster> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[NonceScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_nonce_score_engine() -> NonceScoreEngine {
    NonceScoreEngine {
        steps: vec![
            Rc::new(PercentNonceScoreAdjuster { percent: 6.6 }) as Rc<dyn NonceScoreAdjuster>,
            Rc::new(TieredNonceScoreAdjuster {
                bounds: vec![1417.0, 2168.0, 3220.0, 4765.0],
                rates: vec![0.89, 1.07, 1.09, 0.88],
            }),
            Rc::new(NonceScoreGraceDaysOffsetAdjuster { weight: 0.6 }),
            Rc::new(RoundingNonceScoreAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 87316.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_score_engine();
        let item = new_nonce_score("code-753", "name-390");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
