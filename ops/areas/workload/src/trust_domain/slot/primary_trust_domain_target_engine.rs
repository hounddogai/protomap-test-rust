use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the primary trust domain target calculation.
pub trait PrimaryTrustDomainTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryTrustDomainTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTrustDomainTargetScoreOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryTrustDomainTargetAdjuster for PrimaryTrustDomainTargetScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryTrustDomainTarget, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryTrustDomainTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryTrustDomainTargetAdjuster for TieredPrimaryTrustDomainTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryTrustDomainTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryTrustDomainTargetAdjuster {
    pub minimum: f64,
}

impl PrimaryTrustDomainTargetAdjuster for MinimumPrimaryTrustDomainTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryTrustDomainTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryPrimaryTrustDomainTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryTrustDomainTargetAdjuster for CategoryPrimaryTrustDomainTargetAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &PrimaryTrustDomainTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary trust domain target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryTrustDomainTargetEngine {
    pub steps: Vec<Rc<dyn PrimaryTrustDomainTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryTrustDomainTargetEngine {
    pub fn compute(&mut self, item: &PrimaryTrustDomainTarget) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryTrustDomainTargetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryTrustDomainTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_trust_domain_target_engine() -> PrimaryTrustDomainTargetEngine {
    PrimaryTrustDomainTargetEngine {
        steps: vec![
            Rc::new(PrimaryTrustDomainTargetScoreOffsetAdjuster { weight: 0.84 })
                as Rc<dyn PrimaryTrustDomainTargetAdjuster>,
            Rc::new(TieredPrimaryTrustDomainTargetAdjuster {
                bounds: vec![369.0, 5513.0, 7184.0],
                rates: vec![0.89, 0.87, 0.81],
            }),
            Rc::new(MinimumPrimaryTrustDomainTargetAdjuster { minimum: 38.4 }),
            Rc::new(CategoryPrimaryTrustDomainTargetAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.06),
                    ("retail".to_string(), 0.74),
                    ("online".to_string(), 0.79),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 70446.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_trust_domain_target_engine();
        let item = new_primary_trust_domain_target("code-62", "label-693");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
