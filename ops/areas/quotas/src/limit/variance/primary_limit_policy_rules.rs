use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary limit policy must pass. Check returns its problems, or none.
pub trait PrimaryLimitPolicyRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryLimitPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitPolicyRequiredExternalRefRule {
    pub message: String,
}

impl PrimaryLimitPolicyRule for PrimaryLimitPolicyRequiredExternalRefRule {
    fn check(&self, item: &PrimaryLimitPolicy) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitPolicyExternalRefLengthRule {
    pub max_length: i64,
}

impl PrimaryLimitPolicyRule for PrimaryLimitPolicyExternalRefLengthRule {
    fn check(&self, item: &PrimaryLimitPolicy) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitPolicyCodePrefixRule {
    pub prefix: String,
}

impl PrimaryLimitPolicyRule for PrimaryLimitPolicyCodePrefixRule {
    fn check(&self, item: &PrimaryLimitPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitPolicyRequiredRegionRule {
    pub message: String,
}

impl PrimaryLimitPolicyRule for PrimaryLimitPolicyRequiredRegionRule {
    fn check(&self, item: &PrimaryLimitPolicy) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary limit policy.
#[derive(Debug, Clone)]
pub struct PrimaryLimitPolicyValidator {
    pub rules: Vec<Rc<dyn PrimaryLimitPolicyRule>>,
}

impl PrimaryLimitPolicyValidator {
    pub fn validate(&self, item: &PrimaryLimitPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryLimitPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryLimitPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_limit_policy_validator() -> PrimaryLimitPolicyValidator {
    PrimaryLimitPolicyValidator {
        rules: vec![
            Rc::new(PrimaryLimitPolicyRequiredExternalRefRule {
                message: "primary limit policy needs this field".to_string(),
            }) as Rc<dyn PrimaryLimitPolicyRule>,
            Rc::new(PrimaryLimitPolicyExternalRefLengthRule { max_length: 32 }),
            Rc::new(PrimaryLimitPolicyCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PrimaryLimitPolicyRequiredRegionRule {
                message: "primary limit policy needs a value".to_string(),
            }),
        ],
    }
}
