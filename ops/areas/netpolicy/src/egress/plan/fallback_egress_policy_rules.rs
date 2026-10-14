use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback egress policy must pass. Check returns its problems, or none.
pub trait FallbackEgressPolicyRule: std::fmt::Debug {
    fn check(&self, item: &FallbackEgressPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressPolicyVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackEgressPolicyRule for FallbackEgressPolicyVersionRangeRule {
    fn check(&self, item: &FallbackEgressPolicy) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressPolicyLocaleLengthRule {
    pub max_length: i64,
}

impl FallbackEgressPolicyRule for FallbackEgressPolicyLocaleLengthRule {
    fn check(&self, item: &FallbackEgressPolicy) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressPolicyRequiredCodeRule {
    pub message: String,
}

impl FallbackEgressPolicyRule for FallbackEgressPolicyRequiredCodeRule {
    fn check(&self, item: &FallbackEgressPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressPolicyCodePrefixRule {
    pub prefix: String,
}

impl FallbackEgressPolicyRule for FallbackEgressPolicyCodePrefixRule {
    fn check(&self, item: &FallbackEgressPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback egress policy.
#[derive(Debug, Clone)]
pub struct FallbackEgressPolicyValidator {
    pub rules: Vec<Rc<dyn FallbackEgressPolicyRule>>,
}

impl FallbackEgressPolicyValidator {
    pub fn validate(&self, item: &FallbackEgressPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackEgressPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackEgressPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_egress_policy_validator() -> FallbackEgressPolicyValidator {
    FallbackEgressPolicyValidator {
        rules: vec![
            Rc::new(FallbackEgressPolicyVersionRangeRule { min: 1000, max: 86400 }) as Rc<dyn FallbackEgressPolicyRule>,
            Rc::new(FallbackEgressPolicyLocaleLengthRule { max_length: 16 }),
            Rc::new(FallbackEgressPolicyRequiredCodeRule {
                message: "fallback egress policy needs an entry".to_string(),
            }),
            Rc::new(FallbackEgressPolicyCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
