use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered silence policy must pass. Check returns its problems, or none.
pub trait TieredSilencePolicyRule: std::fmt::Debug {
    fn check(&self, item: &TieredSilencePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilencePolicyRequiredTenantIdRule {
    pub message: String,
}

impl TieredSilencePolicyRule for TieredSilencePolicyRequiredTenantIdRule {
    fn check(&self, item: &TieredSilencePolicy) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilencePolicyRequiredNameRule {
    pub message: String,
}

impl TieredSilencePolicyRule for TieredSilencePolicyRequiredNameRule {
    fn check(&self, item: &TieredSilencePolicy) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilencePolicyWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredSilencePolicyRule for TieredSilencePolicyWeightRangeRule {
    fn check(&self, item: &TieredSilencePolicy) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered silence policy.
#[derive(Debug, Clone)]
pub struct TieredSilencePolicyValidator {
    pub rules: Vec<Rc<dyn TieredSilencePolicyRule>>,
}

impl TieredSilencePolicyValidator {
    pub fn validate(&self, item: &TieredSilencePolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredSilencePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredSilencePolicy]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_tiered_silence_policy_validator() -> TieredSilencePolicyValidator {
    TieredSilencePolicyValidator {
        rules: vec![
            Rc::new(TieredSilencePolicyRequiredTenantIdRule {
                message: "tiered silence policy needs this field".to_string(),
            }) as Rc<dyn TieredSilencePolicyRule>,
            Rc::new(TieredSilencePolicyRequiredNameRule {
                message: "tiered silence policy needs an entry".to_string(),
            }),
            Rc::new(TieredSilencePolicyWeightRangeRule { min: 5.5, max: 4823.5 }),
        ],
    }
}
