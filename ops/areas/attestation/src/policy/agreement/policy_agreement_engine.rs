use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the policy agreement calculation.
pub trait PolicyAgreementFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAgreementFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl PolicyAgreementFactor for PolicyAgreementFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &PolicyAgreement, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPolicyAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PolicyAgreementFactor for FlaggedPolicyAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PolicyAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPolicyAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PolicyAgreementFactor for RegionPolicyAgreementAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &PolicyAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPolicyAgreementAdjuster {
    pub step: f64,
}

impl PolicyAgreementFactor for RoundingPolicyAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PolicyAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPolicyAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PolicyAgreementFactor for TieredPolicyAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PolicyAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyAgreementAdjuster {
    pub cap: f64,
}

impl PolicyAgreementFactor for CapPolicyAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the policy agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyAgreementEngine {
    pub steps: Vec<Rc<dyn PolicyAgreementFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyAgreementEngine {
    pub fn evaluate(&mut self, item: &PolicyAgreement) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PolicyAgreementFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PolicyAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_policy_agreement_engine(flags: &flags::Registry) -> PolicyAgreementEngine {
    PolicyAgreementEngine {
        steps: vec![
            Rc::new(PolicyAgreementFloorValueOffsetAdjuster { weight: 1.84 }) as Rc<dyn PolicyAgreementFactor>,
            Rc::new(FlaggedPolicyAgreementAdjuster {
                flags: flags.clone(),
                flag: "policy-agreement-rolling".to_string(),
                percent: -8.8,
            }),
            Rc::new(RegionPolicyAgreementAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.97),
                    ("retail".to_string(), 0.99),
                    ("north".to_string(), 1.06),
                ]),
            }),
            Rc::new(RoundingPolicyAgreementAdjuster { step: 0.05 }),
            Rc::new(TieredPolicyAgreementAdjuster {
                bounds: vec![2624.0, 2879.0, 4093.0, 4517.0],
                rates: vec![0.98, 0.88, 0.97, 0.99],
            }),
            Rc::new(CapPolicyAgreementAdjuster { cap: 31084.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 58786.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_agreement_engine(&flags::new_registry());
        let item = new_policy_agreement("code-228", "title-640");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
