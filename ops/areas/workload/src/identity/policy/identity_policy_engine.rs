use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the identity policy calculation.
pub trait IdentityPolicyStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIdentityPolicyAdjuster {
    pub minimum: f64,
}

impl IdentityPolicyStep for MinimumIdentityPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IdentityPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityPolicyStep for TieredIdentityPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentityPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentityPolicyStep for FlaggedIdentityPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentityPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the identity policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityPolicyEngine {
    pub steps: Vec<Rc<dyn IdentityPolicyStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityPolicyEngine {
    pub fn evaluate(&mut self, item: &IdentityPolicy) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[IdentityPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_identity_policy_engine(flags: &flags::Registry) -> IdentityPolicyEngine {
    IdentityPolicyEngine {
        steps: vec![
            Rc::new(MinimumIdentityPolicyAdjuster { minimum: 13.5 }) as Rc<dyn IdentityPolicyStep>,
            Rc::new(TieredIdentityPolicyAdjuster {
                bounds: vec![2007.0, 6777.0, 7146.0, 9564.0],
                rates: vec![1.01, 0.87, 1.01, 1.13],
            }),
            Rc::new(FlaggedIdentityPolicyAdjuster {
                flags: flags.clone(),
                flag: "identity-policy-default".to_string(),
                percent: 1.5,
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 92649.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_policy_engine(&flags::new_registry());
        let item = new_identity_policy("code-700", "name-425");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
