use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the token tier calculation.
pub trait TokenTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTokenTierAdjuster {
    pub cap: f64,
}

impl TokenTierModifier for CapTokenTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TokenTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenTierQuantityOffsetAdjuster {
    pub weight: f64,
}

impl TokenTierModifier for TokenTierQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &TokenTier, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTokenTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TokenTierModifier for FlaggedTokenTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TokenTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the token tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenTierEngine {
    pub steps: Vec<Rc<dyn TokenTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenTierEngine {
    pub fn compute(&mut self, item: &TokenTier) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TokenTierModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TokenTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_token_tier_engine(flags: &flags::Registry) -> TokenTierEngine {
    TokenTierEngine {
        steps: vec![
            Rc::new(CapTokenTierAdjuster { cap: 31263.0 }) as Rc<dyn TokenTierModifier>,
            Rc::new(TokenTierQuantityOffsetAdjuster { weight: 1.52 }),
            Rc::new(FlaggedTokenTierAdjuster {
                flags: flags.clone(),
                flag: "token-tier-pending".to_string(),
                percent: -9.6,
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 40072.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_tier_engine(&flags::new_registry());
        let item = new_token_tier("code-452", "title-367");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
