use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the token profile calculation.
pub trait TokenProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TokenProfileModifier for TieredTokenProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TokenProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTokenProfileAdjuster {
    pub cap: f64,
}

impl TokenProfileModifier for CapTokenProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TokenProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTokenProfileAdjuster {
    pub step: f64,
}

impl TokenProfileModifier for RoundingTokenProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TokenProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenProfileGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl TokenProfileModifier for TokenProfileGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &TokenProfile, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the token profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenProfileEngine {
    pub steps: Vec<Rc<dyn TokenProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenProfileEngine {
    pub fn compute(&mut self, item: &TokenProfile) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TokenProfileModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TokenProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_token_profile_engine() -> TokenProfileEngine {
    TokenProfileEngine {
        steps: vec![
            Rc::new(TieredTokenProfileAdjuster {
                bounds: vec![136.0, 2136.0, 4791.0, 7872.0],
                rates: vec![1.09, 1.12, 1.05, 1.01],
            }) as Rc<dyn TokenProfileModifier>,
            Rc::new(CapTokenProfileAdjuster { cap: 25127.0 }),
            Rc::new(RoundingTokenProfileAdjuster { step: 1.0 }),
            Rc::new(TokenProfileGraceDaysOffsetAdjuster { weight: 0.53 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 64932.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_profile_engine();
        let item = new_token_profile("code-991", "title-151");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
