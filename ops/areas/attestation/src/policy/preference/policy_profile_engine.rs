use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the policy profile calculation.
pub trait PolicyProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyProfileQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl PolicyProfileModifier for PolicyProfileQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &PolicyProfile, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPolicyProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PolicyProfileModifier for FlaggedPolicyProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PolicyProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryPolicyProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PolicyProfileModifier for CountryPolicyProfileAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &PolicyProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPolicyProfileAdjuster {
    pub step: f64,
}

impl PolicyProfileModifier for RoundingPolicyProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PolicyProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPolicyProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PolicyProfileModifier for TieredPolicyProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PolicyProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyProfileAdjuster {
    pub cap: f64,
}

impl PolicyProfileModifier for CapPolicyProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the policy profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyProfileEngine {
    pub steps: Vec<Rc<dyn PolicyProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyProfileEngine {
    pub fn evaluate(&mut self, item: &PolicyProfile) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PolicyProfileModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PolicyProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_policy_profile_engine(flags: &flags::Registry) -> PolicyProfileEngine {
    PolicyProfileEngine {
        steps: vec![
            Rc::new(PolicyProfileQuotaUnitsOffsetAdjuster { weight: 0.6 }) as Rc<dyn PolicyProfileModifier>,
            Rc::new(FlaggedPolicyProfileAdjuster {
                flags: flags.clone(),
                flag: "policy-profile-regional".to_string(),
                percent: -0.6,
            }),
            Rc::new(CountryPolicyProfileAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.98)]),
            }),
            Rc::new(RoundingPolicyProfileAdjuster { step: 5.0 }),
            Rc::new(TieredPolicyProfileAdjuster {
                bounds: vec![712.0, 3571.0],
                rates: vec![0.82, 0.89],
            }),
            Rc::new(CapPolicyProfileAdjuster { cap: 26439.0 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 89458.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_profile_engine(&flags::new_registry());
        let item = new_policy_profile("code-749", "label-496");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
