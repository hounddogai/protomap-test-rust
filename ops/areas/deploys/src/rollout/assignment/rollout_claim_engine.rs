use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rollout claim calculation.
pub trait RolloutClaimFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutClaimPriorityOffsetAdjuster {
    pub weight: f64,
}

impl RolloutClaimFactor for RolloutClaimPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &RolloutClaim, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRolloutClaimAdjuster {
    pub step: f64,
}

impl RolloutClaimFactor for RoundingRolloutClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RolloutClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RolloutClaimFactor for TieredRolloutClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RolloutClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rollout claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutClaimEngine {
    pub steps: Vec<Rc<dyn RolloutClaimFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutClaimEngine {
    pub fn evaluate(&mut self, item: &RolloutClaim) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RolloutClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rollout_claim_engine() -> RolloutClaimEngine {
    RolloutClaimEngine {
        steps: vec![
            Rc::new(RolloutClaimPriorityOffsetAdjuster { weight: 1.23 }) as Rc<dyn RolloutClaimFactor>,
            Rc::new(RoundingRolloutClaimAdjuster { step: 0.1 }),
            Rc::new(TieredRolloutClaimAdjuster {
                bounds: vec![2435.0, 4472.0, 8244.0, 9398.0],
                rates: vec![1.03, 1.07, 0.85, 1.07],
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 15332.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_claim_engine();
        let item = new_rollout_claim("code-504", "title-603");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
