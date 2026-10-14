use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rule item must pass. Check returns its problems, or none.
pub trait RuleItemRule: std::fmt::Debug {
    fn check(&self, item: &RuleItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleItemTitleLengthRule {
    pub max_length: i64,
}

impl RuleItemRule for RuleItemTitleLengthRule {
    fn check(&self, item: &RuleItem) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleItemAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RuleItemRule for RuleItemAllowedTitleRule {
    fn check(&self, item: &RuleItem) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleItemCodeLengthRule {
    pub max_length: i64,
}

impl RuleItemRule for RuleItemCodeLengthRule {
    fn check(&self, item: &RuleItem) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleItemPinnedGuardRule {
    pub limit: f64,
}

impl RuleItemRule for RuleItemPinnedGuardRule {
    fn check(&self, item: &RuleItem) -> Vec<String> {
        if item.pinned && (item.cost > self.limit) {
            return vec![format!("cost {} exceeds {} while pinned", item.cost, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleItemRequiredCodeRule {
    pub message: String,
}

impl RuleItemRule for RuleItemRequiredCodeRule {
    fn check(&self, item: &RuleItem) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleItemCodePrefixRule {
    pub prefix: String,
}

impl RuleItemRule for RuleItemCodePrefixRule {
    fn check(&self, item: &RuleItem) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule item.
#[derive(Debug, Clone)]
pub struct RuleItemValidator {
    pub rules: Vec<Rc<dyn RuleItemRule>>,
}

impl RuleItemValidator {
    pub fn validate(&self, item: &RuleItem) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_item_validator() -> RuleItemValidator {
    RuleItemValidator {
        rules: vec![
            Rc::new(RuleItemTitleLengthRule { max_length: 3600 }) as Rc<dyn RuleItemRule>,
            Rc::new(RuleItemAllowedTitleRule {
                allowed: vec!["fallback".to_string()],
            }),
            Rc::new(RuleItemCodeLengthRule { max_length: 64 }),
            Rc::new(RuleItemPinnedGuardRule { limit: 9.1 }),
            Rc::new(RuleItemRequiredCodeRule {
                message: "rule item needs this field".to_string(),
            }),
            Rc::new(RuleItemCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}
