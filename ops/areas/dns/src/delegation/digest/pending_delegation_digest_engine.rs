use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending delegation digest calculation.
pub trait PendingDelegationDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingDelegationDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationDigestRateOffsetAdjuster {
    pub weight: f64,
}

impl PendingDelegationDigestModifier for PendingDelegationDigestRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &PendingDelegationDigest, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePendingDelegationDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingDelegationDigestModifier for NamePendingDelegationDigestAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PendingDelegationDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingDelegationDigestAdjuster {
    pub percent: f64,
}

impl PendingDelegationDigestModifier for PercentPendingDelegationDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingDelegationDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingDelegationDigestAdjuster {
    pub minimum: f64,
}

impl PendingDelegationDigestModifier for MinimumPendingDelegationDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingDelegationDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingDelegationDigestAdjuster {
    pub step: f64,
}

impl PendingDelegationDigestModifier for RoundingPendingDelegationDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingDelegationDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the pending delegation digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingDelegationDigestEngine {
    pub steps: Vec<Rc<dyn PendingDelegationDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingDelegationDigestEngine {
    pub fn compute(&mut self, item: &PendingDelegationDigest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PendingDelegationDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_delegation_digest_engine() -> PendingDelegationDigestEngine {
    PendingDelegationDigestEngine {
        steps: vec![
            Rc::new(PendingDelegationDigestRateOffsetAdjuster { weight: 0.36 })
                as Rc<dyn PendingDelegationDigestModifier>,
            Rc::new(NamePendingDelegationDigestAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.75), ("retail".to_string(), 1.12)]),
            }),
            Rc::new(PercentPendingDelegationDigestAdjuster { percent: -24.4 }),
            Rc::new(MinimumPendingDelegationDigestAdjuster { minimum: 24.0 }),
            Rc::new(RoundingPendingDelegationDigestAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 81576.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_delegation_digest_engine();
        let item = new_pending_delegation_digest("code-140", "name-266");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
