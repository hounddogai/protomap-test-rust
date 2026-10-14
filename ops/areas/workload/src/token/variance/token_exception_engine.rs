use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the token exception calculation.
pub trait TokenExceptionFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTokenExceptionAdjuster {
    pub cap: f64,
}

impl TokenExceptionFactor for CapTokenExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TokenException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenExceptionAmountOffsetAdjuster {
    pub weight: f64,
}

impl TokenExceptionFactor for TokenExceptionAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &TokenException, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeTokenExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TokenExceptionFactor for ScopeTokenExceptionAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &TokenException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the token exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenExceptionEngine {
    pub steps: Vec<Rc<dyn TokenExceptionFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenExceptionEngine {
    pub fn compute(&mut self, item: &TokenException) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TokenExceptionFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TokenException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_token_exception_engine() -> TokenExceptionEngine {
    TokenExceptionEngine {
        steps: vec![
            Rc::new(CapTokenExceptionAdjuster { cap: 43041.0 }) as Rc<dyn TokenExceptionFactor>,
            Rc::new(TokenExceptionAmountOffsetAdjuster { weight: 0.81 }),
            Rc::new(ScopeTokenExceptionAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.22)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 96556.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_exception_engine();
        let item = new_token_exception("code-663", "name-125");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
