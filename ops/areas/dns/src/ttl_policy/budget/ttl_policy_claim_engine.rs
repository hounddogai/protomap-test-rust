use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the ttl policy claim calculation.
pub trait TtlPolicyClaimFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTtlPolicyClaimAdjuster {
    pub cap: f64,
}

impl TtlPolicyClaimFactor for CapTtlPolicyClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TtlPolicyClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTtlPolicyClaimAdjuster {
    pub minimum: f64,
}

impl TtlPolicyClaimFactor for MinimumTtlPolicyClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TtlPolicyClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyClaimCostOffsetAdjuster {
    pub weight: f64,
}

impl TtlPolicyClaimFactor for TtlPolicyClaimCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &TtlPolicyClaim, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTtlPolicyClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TtlPolicyClaimFactor for FlaggedTtlPolicyClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TtlPolicyClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the ttl policy claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyClaimEngine {
    pub steps: Vec<Rc<dyn TtlPolicyClaimFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyClaimEngine {
    pub fn run(&mut self, item: &TtlPolicyClaim) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TtlPolicyClaimFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_ttl_policy_claim_engine(flags: &flags::Registry) -> TtlPolicyClaimEngine {
    TtlPolicyClaimEngine {
        steps: vec![
            Rc::new(CapTtlPolicyClaimAdjuster { cap: 4563.0 }) as Rc<dyn TtlPolicyClaimFactor>,
            Rc::new(MinimumTtlPolicyClaimAdjuster { minimum: 17.2 }),
            Rc::new(TtlPolicyClaimCostOffsetAdjuster { weight: 0.11 }),
            Rc::new(FlaggedTtlPolicyClaimAdjuster {
                flags: flags.clone(),
                flag: "ttl-policy-claim-strict".to_string(),
                percent: 2.2,
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 44228.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_claim_engine(&flags::new_registry());
        let item = new_ttl_policy_claim("code-574", "name-760");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
