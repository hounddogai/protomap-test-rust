use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the token claim calculation.
pub trait TokenClaimModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTokenClaimAdjuster {
    pub percent: f64,
}

impl TokenClaimModifier for PercentTokenClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TokenClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTokenClaimAdjuster {
    pub step: f64,
}

impl TokenClaimModifier for RoundingTokenClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TokenClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenClaimExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl TokenClaimModifier for TokenClaimExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &TokenClaim, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

/// Runs the token claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenClaimEngine {
    pub steps: Vec<Rc<dyn TokenClaimModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenClaimEngine {
    pub fn compute(&mut self, item: &TokenClaim) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TokenClaimModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TokenClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_token_claim_engine() -> TokenClaimEngine {
    TokenClaimEngine {
        steps: vec![
            Rc::new(PercentTokenClaimAdjuster { percent: 8.9 }) as Rc<dyn TokenClaimModifier>,
            Rc::new(RoundingTokenClaimAdjuster { step: 0.5 }),
            Rc::new(TokenClaimExpiresAtOffsetAdjuster { weight: 1.68 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 36755.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_claim_engine();
        let item = new_token_claim("code-746", "name-185");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
