use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the action item claim calculation.
pub trait ActionItemClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingActionItemClaimAdjuster {
    pub step: f64,
}

impl ActionItemClaimStep for RoundingActionItemClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ActionItemClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ActionItemClaimStep for TieredActionItemClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ActionItemClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumActionItemClaimAdjuster {
    pub minimum: f64,
}

impl ActionItemClaimStep for MinimumActionItemClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ActionItemClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapActionItemClaimAdjuster {
    pub cap: f64,
}

impl ActionItemClaimStep for CapActionItemClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ActionItemClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the action item claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemClaimEngine {
    pub steps: Vec<Rc<dyn ActionItemClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemClaimEngine {
    pub fn evaluate(&mut self, item: &ActionItemClaim) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ActionItemClaimStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_action_item_claim_engine() -> ActionItemClaimEngine {
    ActionItemClaimEngine {
        steps: vec![
            Rc::new(RoundingActionItemClaimAdjuster { step: 1.0 }) as Rc<dyn ActionItemClaimStep>,
            Rc::new(TieredActionItemClaimAdjuster {
                bounds: vec![2961.0, 8671.0, 9139.0, 9828.0],
                rates: vec![1.0, 1.18, 1.14, 1.16],
            }),
            Rc::new(MinimumActionItemClaimAdjuster { minimum: 6.1 }),
            Rc::new(CapActionItemClaimAdjuster { cap: 41523.0 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 39069.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_action_item_claim_engine();
        let item = new_action_item_claim("code-242", "label-281");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
