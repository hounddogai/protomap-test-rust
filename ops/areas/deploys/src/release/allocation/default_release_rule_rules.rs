use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default release rule must pass. Check returns its problems, or none.
pub trait DefaultReleaseRuleRule: std::fmt::Debug {
    fn check(&self, item: &DefaultReleaseRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseRuleReviewerLengthRule {
    pub max_length: i64,
}

impl DefaultReleaseRuleRule for DefaultReleaseRuleReviewerLengthRule {
    fn check(&self, item: &DefaultReleaseRule) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseRuleScopeLengthRule {
    pub max_length: i64,
}

impl DefaultReleaseRuleRule for DefaultReleaseRuleScopeLengthRule {
    fn check(&self, item: &DefaultReleaseRule) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseRuleRequiredScopeRule {
    pub message: String,
}

impl DefaultReleaseRuleRule for DefaultReleaseRuleRequiredScopeRule {
    fn check(&self, item: &DefaultReleaseRule) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseRuleRequiredCodeRule {
    pub message: String,
}

impl DefaultReleaseRuleRule for DefaultReleaseRuleRequiredCodeRule {
    fn check(&self, item: &DefaultReleaseRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseRuleCodeLengthRule {
    pub max_length: i64,
}

impl DefaultReleaseRuleRule for DefaultReleaseRuleCodeLengthRule {
    fn check(&self, item: &DefaultReleaseRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default release rule.
#[derive(Debug, Clone)]
pub struct DefaultReleaseRuleValidator {
    pub rules: Vec<Rc<dyn DefaultReleaseRuleRule>>,
}

impl DefaultReleaseRuleValidator {
    pub fn validate(&self, item: &DefaultReleaseRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultReleaseRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultReleaseRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_release_rule_validator() -> DefaultReleaseRuleValidator {
    DefaultReleaseRuleValidator {
        rules: vec![
            Rc::new(DefaultReleaseRuleReviewerLengthRule { max_length: 64 }) as Rc<dyn DefaultReleaseRuleRule>,
            Rc::new(DefaultReleaseRuleScopeLengthRule { max_length: 1000 }),
            Rc::new(DefaultReleaseRuleRequiredScopeRule {
                message: "default release rule needs an entry".to_string(),
            }),
            Rc::new(DefaultReleaseRuleRequiredCodeRule {
                message: "default release rule needs a value".to_string(),
            }),
            Rc::new(DefaultReleaseRuleCodeLengthRule { max_length: 86400 }),
        ],
    }
}
