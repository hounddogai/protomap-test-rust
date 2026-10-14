use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the trust domain override calculation.
pub trait TrustDomainOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTrustDomainOverrideAdjuster {
    pub cap: f64,
}

impl TrustDomainOverrideAdjuster for CapTrustDomainOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TrustDomainOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainOverrideMinimumOffsetAdjuster {
    pub weight: f64,
}

impl TrustDomainOverrideAdjuster for TrustDomainOverrideMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &TrustDomainOverride, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainOverrideAdjuster for TieredTrustDomainOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceTrustDomainOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TrustDomainOverrideAdjuster for SourceTrustDomainOverrideAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &TrustDomainOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the trust domain override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainOverrideEngine {
    pub steps: Vec<Rc<dyn TrustDomainOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainOverrideEngine {
    pub fn compute(&mut self, item: &TrustDomainOverride) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainOverrideAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_trust_domain_override_engine() -> TrustDomainOverrideEngine {
    TrustDomainOverrideEngine {
        steps: vec![
            Rc::new(CapTrustDomainOverrideAdjuster { cap: 32890.0 }) as Rc<dyn TrustDomainOverrideAdjuster>,
            Rc::new(TrustDomainOverrideMinimumOffsetAdjuster { weight: 0.85 }),
            Rc::new(TieredTrustDomainOverrideAdjuster {
                bounds: vec![2467.0, 3610.0],
                rates: vec![1.07, 0.91],
            }),
            Rc::new(SourceTrustDomainOverrideAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.99), ("west".to_string(), 1.13)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 95580.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_override_engine();
        let item = new_trust_domain_override("code-864", "title-901");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
