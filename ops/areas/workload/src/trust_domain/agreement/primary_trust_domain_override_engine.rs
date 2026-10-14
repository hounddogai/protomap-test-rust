use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary trust domain override calculation.
pub trait PrimaryTrustDomainOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryTrustDomainOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryTrustDomainOverrideAdjuster {
    pub step: f64,
}

impl PrimaryTrustDomainOverrideModifier for RoundingPrimaryTrustDomainOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryTrustDomainOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPrimaryTrustDomainOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryTrustDomainOverrideModifier for RegionPrimaryTrustDomainOverrideAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &PrimaryTrustDomainOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryTrustDomainOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryTrustDomainOverrideModifier for TieredPrimaryTrustDomainOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryTrustDomainOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the primary trust domain override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryTrustDomainOverrideEngine {
    pub steps: Vec<Rc<dyn PrimaryTrustDomainOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryTrustDomainOverrideEngine {
    pub fn run(&mut self, item: &PrimaryTrustDomainOverride) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryTrustDomainOverrideModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryTrustDomainOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_trust_domain_override_engine() -> PrimaryTrustDomainOverrideEngine {
    PrimaryTrustDomainOverrideEngine {
        steps: vec![
            Rc::new(RoundingPrimaryTrustDomainOverrideAdjuster { step: 0.1 })
                as Rc<dyn PrimaryTrustDomainOverrideModifier>,
            Rc::new(RegionPrimaryTrustDomainOverrideAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.39),
                    ("central".to_string(), 1.12),
                    ("west".to_string(), 0.77),
                ]),
            }),
            Rc::new(TieredPrimaryTrustDomainOverrideAdjuster {
                bounds: vec![5915.0, 6613.0, 6754.0, 9152.0],
                rates: vec![0.9, 1.19, 0.91, 0.99],
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 39462.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_trust_domain_override_engine();
        let item = new_primary_trust_domain_override("code-34", "label-861");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
