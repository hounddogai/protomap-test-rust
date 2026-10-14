use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy egress rollup must pass. Check returns its problems, or none.
pub trait LegacyEgressRollupRule: std::fmt::Debug {
    fn check(&self, item: &LegacyEgressRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEgressRollupCodeLengthRule {
    pub max_length: i64,
}

impl LegacyEgressRollupRule for LegacyEgressRollupCodeLengthRule {
    fn check(&self, item: &LegacyEgressRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEgressRollupRequiredCodeRule {
    pub message: String,
}

impl LegacyEgressRollupRule for LegacyEgressRollupRequiredCodeRule {
    fn check(&self, item: &LegacyEgressRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEgressRollupNameLengthRule {
    pub max_length: i64,
}

impl LegacyEgressRollupRule for LegacyEgressRollupNameLengthRule {
    fn check(&self, item: &LegacyEgressRollup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEgressRollupCodePrefixRule {
    pub prefix: String,
}

impl LegacyEgressRollupRule for LegacyEgressRollupCodePrefixRule {
    fn check(&self, item: &LegacyEgressRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEgressRollupRequiredRegionRule {
    pub message: String,
}

impl LegacyEgressRollupRule for LegacyEgressRollupRequiredRegionRule {
    fn check(&self, item: &LegacyEgressRollup) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy egress rollup.
#[derive(Debug, Clone)]
pub struct LegacyEgressRollupValidator {
    pub rules: Vec<Rc<dyn LegacyEgressRollupRule>>,
}

impl LegacyEgressRollupValidator {
    pub fn validate(&self, item: &LegacyEgressRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyEgressRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyEgressRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_egress_rollup_validator() -> LegacyEgressRollupValidator {
    LegacyEgressRollupValidator {
        rules: vec![
            Rc::new(LegacyEgressRollupCodeLengthRule { max_length: 80 }) as Rc<dyn LegacyEgressRollupRule>,
            Rc::new(LegacyEgressRollupRequiredCodeRule {
                message: "legacy egress rollup needs an entry".to_string(),
            }),
            Rc::new(LegacyEgressRollupNameLengthRule { max_length: 86400 }),
            Rc::new(LegacyEgressRollupCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(LegacyEgressRollupRequiredRegionRule {
                message: "legacy egress rollup needs this field".to_string(),
            }),
        ],
    }
}
