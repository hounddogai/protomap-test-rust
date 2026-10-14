use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict trust domain target calculation.
pub trait StrictTrustDomainTargetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictTrustDomainTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameStrictTrustDomainTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictTrustDomainTargetModifier for NameStrictTrustDomainTargetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &StrictTrustDomainTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictTrustDomainTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictTrustDomainTargetModifier for TieredStrictTrustDomainTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictTrustDomainTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTrustDomainTargetRateOffsetAdjuster {
    pub weight: f64,
}

impl StrictTrustDomainTargetModifier for StrictTrustDomainTargetRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &StrictTrustDomainTarget, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictTrustDomainTargetAdjuster {
    pub step: f64,
}

impl StrictTrustDomainTargetModifier for RoundingStrictTrustDomainTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictTrustDomainTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the strict trust domain target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictTrustDomainTargetEngine {
    pub steps: Vec<Rc<dyn StrictTrustDomainTargetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictTrustDomainTargetEngine {
    pub fn evaluate(&mut self, item: &StrictTrustDomainTarget) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[StrictTrustDomainTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_trust_domain_target_engine() -> StrictTrustDomainTargetEngine {
    StrictTrustDomainTargetEngine {
        steps: vec![
            Rc::new(NameStrictTrustDomainTargetAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.34),
                    ("online".to_string(), 1.25),
                    ("central".to_string(), 1.23),
                ]),
            }) as Rc<dyn StrictTrustDomainTargetModifier>,
            Rc::new(TieredStrictTrustDomainTargetAdjuster {
                bounds: vec![7139.0, 9813.0],
                rates: vec![0.97, 1.13],
            }),
            Rc::new(StrictTrustDomainTargetRateOffsetAdjuster { weight: 1.09 }),
            Rc::new(RoundingStrictTrustDomainTargetAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 14005.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_trust_domain_target_engine();
        let item = new_strict_trust_domain_target("code-697", "name-690");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
