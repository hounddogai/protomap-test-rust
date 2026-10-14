use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the rolling delegation claim calculation.
pub trait RollingDelegationClaimModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingDelegationClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingDelegationClaimAdjuster {
    pub cap: f64,
}

impl RollingDelegationClaimModifier for CapRollingDelegationClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingDelegationClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingDelegationClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingDelegationClaimModifier for TieredRollingDelegationClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingDelegationClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingDelegationClaimAdjuster {
    pub percent: f64,
}

impl RollingDelegationClaimModifier for PercentRollingDelegationClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingDelegationClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling delegation claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingDelegationClaimEngine {
    pub steps: Vec<Rc<dyn RollingDelegationClaimModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingDelegationClaimEngine {
    pub fn evaluate(&mut self, item: &RollingDelegationClaim) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingDelegationClaimModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingDelegationClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_delegation_claim_engine() -> RollingDelegationClaimEngine {
    RollingDelegationClaimEngine {
        steps: vec![
            Rc::new(CapRollingDelegationClaimAdjuster { cap: 8478.0 }) as Rc<dyn RollingDelegationClaimModifier>,
            Rc::new(TieredRollingDelegationClaimAdjuster {
                bounds: vec![3155.0, 7944.0, 8035.0],
                rates: vec![1.01, 1.03, 1.11],
            }),
            Rc::new(PercentRollingDelegationClaimAdjuster { percent: 16.7 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 57707.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_delegation_claim_engine();
        let item = new_rolling_delegation_claim("code-913", "title-469");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
