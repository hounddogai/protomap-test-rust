use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist target must pass. Check returns its problems, or none.
pub trait AllowlistTargetRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTargetCodeLengthRule {
    pub max_length: i64,
}

impl AllowlistTargetRule for AllowlistTargetCodeLengthRule {
    fn check(&self, item: &AllowlistTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTargetCurrencyLengthRule {
    pub max_length: i64,
}

impl AllowlistTargetRule for AllowlistTargetCurrencyLengthRule {
    fn check(&self, item: &AllowlistTarget) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTargetCodePrefixRule {
    pub prefix: String,
}

impl AllowlistTargetRule for AllowlistTargetCodePrefixRule {
    fn check(&self, item: &AllowlistTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTargetMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AllowlistTargetRule for AllowlistTargetMinimumRangeRule {
    fn check(&self, item: &AllowlistTarget) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTargetTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl AllowlistTargetRule for AllowlistTargetTagsLimitRule {
    fn check(&self, item: &AllowlistTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTargetLabelLengthRule {
    pub max_length: i64,
}

impl AllowlistTargetRule for AllowlistTargetLabelLengthRule {
    fn check(&self, item: &AllowlistTarget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist target.
#[derive(Debug, Clone)]
pub struct AllowlistTargetValidator {
    pub rules: Vec<Rc<dyn AllowlistTargetRule>>,
}

impl AllowlistTargetValidator {
    pub fn validate(&self, item: &AllowlistTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_target_validator() -> AllowlistTargetValidator {
    AllowlistTargetValidator {
        rules: vec![
            Rc::new(AllowlistTargetCodeLengthRule { max_length: 255 }) as Rc<dyn AllowlistTargetRule>,
            Rc::new(AllowlistTargetCurrencyLengthRule { max_length: 32 }),
            Rc::new(AllowlistTargetCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(AllowlistTargetMinimumRangeRule { min: 6.8, max: 835.4 }),
            Rc::new(AllowlistTargetTagsLimitRule {
                max_count: 255,
                banned: vec!["capped".to_string()],
            }),
            Rc::new(AllowlistTargetLabelLengthRule { max_length: 86400 }),
        ],
    }
}
