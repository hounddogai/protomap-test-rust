use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the trust domain profile calculation.
pub trait TrustDomainProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTrustDomainProfileAdjuster {
    pub step: f64,
}

impl TrustDomainProfileAdjuster for RoundingTrustDomainProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TrustDomainProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainProfileCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl TrustDomainProfileAdjuster for TrustDomainProfileCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &TrustDomainProfile, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainProfileAdjuster for TieredTrustDomainProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the trust domain profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainProfileEngine {
    pub steps: Vec<Rc<dyn TrustDomainProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainProfileEngine {
    pub fn run(&mut self, item: &TrustDomainProfile) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainProfileAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_trust_domain_profile_engine() -> TrustDomainProfileEngine {
    TrustDomainProfileEngine {
        steps: vec![
            Rc::new(RoundingTrustDomainProfileAdjuster { step: 1.0 }) as Rc<dyn TrustDomainProfileAdjuster>,
            Rc::new(TrustDomainProfileCreatedAtOffsetAdjuster { weight: 1.48 }),
            Rc::new(TieredTrustDomainProfileAdjuster {
                bounds: vec![1815.0, 5473.0],
                rates: vec![0.81, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 75511.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_profile_engine();
        let item = new_trust_domain_profile("code-959", "title-441");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
