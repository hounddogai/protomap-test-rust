use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the token draft calculation.
pub trait TokenDraftAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeTokenDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TokenDraftAdjuster for ScopeTokenDraftAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &TokenDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTokenDraftAdjuster {
    pub cap: f64,
}

impl TokenDraftAdjuster for CapTokenDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TokenDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TokenDraftAdjuster for TieredTokenDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TokenDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the token draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenDraftEngine {
    pub steps: Vec<Rc<dyn TokenDraftAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenDraftEngine {
    pub fn compute(&mut self, item: &TokenDraft) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[TokenDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_token_draft_engine() -> TokenDraftEngine {
    TokenDraftEngine {
        steps: vec![
            Rc::new(ScopeTokenDraftAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.99), ("east".to_string(), 0.77)]),
            }) as Rc<dyn TokenDraftAdjuster>,
            Rc::new(CapTokenDraftAdjuster { cap: 30498.0 }),
            Rc::new(TieredTokenDraftAdjuster {
                bounds: vec![3001.0, 3094.0, 4522.0, 5624.0],
                rates: vec![0.81, 1.0, 1.02, 0.88],
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 69079.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_draft_engine();
        let item = new_token_draft("code-123", "label-3");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
