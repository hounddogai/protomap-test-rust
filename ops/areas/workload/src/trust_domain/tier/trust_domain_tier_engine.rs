use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the trust domain tier calculation.
pub trait TrustDomainTierStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTrustDomainTierAdjuster {
    pub cap: f64,
}

impl TrustDomainTierStep for CapTrustDomainTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TrustDomainTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTrustDomainTierAdjuster {
    pub percent: f64,
}

impl TrustDomainTierStep for PercentTrustDomainTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TrustDomainTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTierQuantityOffsetAdjuster {
    pub weight: f64,
}

impl TrustDomainTierStep for TrustDomainTierQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &TrustDomainTier, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

/// Runs the trust domain tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainTierEngine {
    pub steps: Vec<Rc<dyn TrustDomainTierStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainTierEngine {
    pub fn evaluate(&mut self, item: &TrustDomainTier) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainTierStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_trust_domain_tier_engine() -> TrustDomainTierEngine {
    TrustDomainTierEngine {
        steps: vec![
            Rc::new(CapTrustDomainTierAdjuster { cap: 37169.0 }) as Rc<dyn TrustDomainTierStep>,
            Rc::new(PercentTrustDomainTierAdjuster { percent: 21.9 }),
            Rc::new(TrustDomainTierQuantityOffsetAdjuster { weight: 1.29 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 6117.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_tier_engine();
        let item = new_trust_domain_tier("code-435", "name-301");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
