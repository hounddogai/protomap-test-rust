use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy limit policy must pass. Check returns its problems, or none.
pub trait LegacyLimitPolicyRule: std::fmt::Debug {
    fn check(&self, item: &LegacyLimitPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitPolicyCodePrefixRule {
    pub prefix: String,
}

impl LegacyLimitPolicyRule for LegacyLimitPolicyCodePrefixRule {
    fn check(&self, item: &LegacyLimitPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitPolicyRequiredCodeRule {
    pub message: String,
}

impl LegacyLimitPolicyRule for LegacyLimitPolicyRequiredCodeRule {
    fn check(&self, item: &LegacyLimitPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitPolicyKeyIdLengthRule {
    pub max_length: i64,
}

impl LegacyLimitPolicyRule for LegacyLimitPolicyKeyIdLengthRule {
    fn check(&self, item: &LegacyLimitPolicy) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy limit policy.
#[derive(Debug, Clone)]
pub struct LegacyLimitPolicyValidator {
    pub rules: Vec<Rc<dyn LegacyLimitPolicyRule>>,
}

impl LegacyLimitPolicyValidator {
    pub fn validate(&self, item: &LegacyLimitPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyLimitPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyLimitPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_limit_policy_validator() -> LegacyLimitPolicyValidator {
    LegacyLimitPolicyValidator {
        rules: vec![
            Rc::new(LegacyLimitPolicyCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn LegacyLimitPolicyRule>,
            Rc::new(LegacyLimitPolicyRequiredCodeRule {
                message: "legacy limit policy needs an entry".to_string(),
            }),
            Rc::new(LegacyLimitPolicyKeyIdLengthRule { max_length: 64 }),
        ],
    }
}
