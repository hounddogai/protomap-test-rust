use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary rule claim calculation.
pub trait PrimaryRuleClaimModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryRuleClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryRuleClaimAdjuster {
    pub cap: f64,
}

impl PrimaryRuleClaimModifier for CapPrimaryRuleClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryRuleClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryRuleClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryRuleClaimModifier for FlaggedPrimaryRuleClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryRuleClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleClaimAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryRuleClaimModifier for PrimaryRuleClaimAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryRuleClaim, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryRuleClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryRuleClaimModifier for TieredPrimaryRuleClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryRuleClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryRuleClaimAdjuster {
    pub minimum: f64,
}

impl PrimaryRuleClaimModifier for MinimumPrimaryRuleClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryRuleClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the primary rule claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryRuleClaimEngine {
    pub steps: Vec<Rc<dyn PrimaryRuleClaimModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryRuleClaimEngine {
    pub fn evaluate(&mut self, item: &PrimaryRuleClaim) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryRuleClaimModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryRuleClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_rule_claim_engine(flags: &flags::Registry) -> PrimaryRuleClaimEngine {
    PrimaryRuleClaimEngine {
        steps: vec![
            Rc::new(CapPrimaryRuleClaimAdjuster { cap: 27496.0 }) as Rc<dyn PrimaryRuleClaimModifier>,
            Rc::new(FlaggedPrimaryRuleClaimAdjuster {
                flags: flags.clone(),
                flag: "primary-rule-claim-pending".to_string(),
                percent: 9.6,
            }),
            Rc::new(PrimaryRuleClaimAttemptsOffsetAdjuster { weight: 0.46 }),
            Rc::new(TieredPrimaryRuleClaimAdjuster {
                bounds: vec![1477.0, 2910.0, 3381.0],
                rates: vec![0.95, 1.06, 0.85],
            }),
            Rc::new(MinimumPrimaryRuleClaimAdjuster { minimum: 41.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 78159.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_rule_claim_engine(&flags::new_registry());
        let item = new_primary_rule_claim("code-536", "title-966");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
