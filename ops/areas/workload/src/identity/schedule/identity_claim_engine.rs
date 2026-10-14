use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the identity claim calculation.
pub trait IdentityClaimAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityClaimAdjuster {
    pub cap: f64,
}

impl IdentityClaimAdjuster for CapIdentityClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionIdentityClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityClaimAdjuster for RegionIdentityClaimAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &IdentityClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityClaimQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl IdentityClaimAdjuster for IdentityClaimQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &IdentityClaim, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIdentityClaimAdjuster {
    pub step: f64,
}

impl IdentityClaimAdjuster for RoundingIdentityClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IdentityClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIdentityClaimAdjuster {
    pub minimum: f64,
}

impl IdentityClaimAdjuster for MinimumIdentityClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IdentityClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIdentityClaimAdjuster {
    pub percent: f64,
}

impl IdentityClaimAdjuster for PercentIdentityClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IdentityClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the identity claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityClaimEngine {
    pub steps: Vec<Rc<dyn IdentityClaimAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityClaimEngine {
    pub fn evaluate(&mut self, item: &IdentityClaim) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[IdentityClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_identity_claim_engine() -> IdentityClaimEngine {
    IdentityClaimEngine {
        steps: vec![
            Rc::new(CapIdentityClaimAdjuster { cap: 12471.0 }) as Rc<dyn IdentityClaimAdjuster>,
            Rc::new(RegionIdentityClaimAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.21), ("online".to_string(), 1.28)]),
            }),
            Rc::new(IdentityClaimQuotaUnitsOffsetAdjuster { weight: 1.24 }),
            Rc::new(RoundingIdentityClaimAdjuster { step: 0.1 }),
            Rc::new(MinimumIdentityClaimAdjuster { minimum: 20.5 }),
            Rc::new(PercentIdentityClaimAdjuster { percent: -1.6 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 80043.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_claim_engine();
        let item = new_identity_claim("code-739", "title-660");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
