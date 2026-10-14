use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant notice must pass. Check returns its problems, or none.
pub trait GrantNoticeRule: std::fmt::Debug {
    fn check(&self, item: &GrantNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantNoticeCodePrefixRule {
    pub prefix: String,
}

impl GrantNoticeRule for GrantNoticeCodePrefixRule {
    fn check(&self, item: &GrantNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantNoticeVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantNoticeRule for GrantNoticeVersionRangeRule {
    fn check(&self, item: &GrantNotice) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantNoticeRequiredNamespaceRule {
    pub message: String,
}

impl GrantNoticeRule for GrantNoticeRequiredNamespaceRule {
    fn check(&self, item: &GrantNotice) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantNoticeNamespaceLengthRule {
    pub max_length: i64,
}

impl GrantNoticeRule for GrantNoticeNamespaceLengthRule {
    fn check(&self, item: &GrantNotice) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantNoticeTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantNoticeRule for GrantNoticeTtlSecondsRangeRule {
    fn check(&self, item: &GrantNotice) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a grant notice.
#[derive(Debug, Clone)]
pub struct GrantNoticeValidator {
    pub rules: Vec<Rc<dyn GrantNoticeRule>>,
}

impl GrantNoticeValidator {
    pub fn validate(&self, item: &GrantNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_notice_validator() -> GrantNoticeValidator {
    GrantNoticeValidator {
        rules: vec![
            Rc::new(GrantNoticeCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn GrantNoticeRule>,
            Rc::new(GrantNoticeVersionRangeRule { min: 32, max: 64 }),
            Rc::new(GrantNoticeRequiredNamespaceRule {
                message: "grant notice needs an entry".to_string(),
            }),
            Rc::new(GrantNoticeNamespaceLengthRule { max_length: 80 }),
            Rc::new(GrantNoticeTtlSecondsRangeRule { min: 16, max: 3600 }),
        ],
    }
}
