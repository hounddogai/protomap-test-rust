use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy policy agreement must pass. Check returns its problems, or none.
pub trait LegacyPolicyAgreementRule: std::fmt::Debug {
    fn check(&self, item: &LegacyPolicyAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyAgreementCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyPolicyAgreementRule for LegacyPolicyAgreementCreatedAtRangeRule {
    fn check(&self, item: &LegacyPolicyAgreement) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyAgreementRequiredCategoryRule {
    pub message: String,
}

impl LegacyPolicyAgreementRule for LegacyPolicyAgreementRequiredCategoryRule {
    fn check(&self, item: &LegacyPolicyAgreement) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyAgreementCategoryLengthRule {
    pub max_length: i64,
}

impl LegacyPolicyAgreementRule for LegacyPolicyAgreementCategoryLengthRule {
    fn check(&self, item: &LegacyPolicyAgreement) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyAgreementTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl LegacyPolicyAgreementRule for LegacyPolicyAgreementTagsLimitRule {
    fn check(&self, item: &LegacyPolicyAgreement) -> Vec<String> {
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

/// Runs every rule of a legacy policy agreement.
#[derive(Debug, Clone)]
pub struct LegacyPolicyAgreementValidator {
    pub rules: Vec<Rc<dyn LegacyPolicyAgreementRule>>,
}

impl LegacyPolicyAgreementValidator {
    pub fn validate(&self, item: &LegacyPolicyAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyPolicyAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyPolicyAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_policy_agreement_validator() -> LegacyPolicyAgreementValidator {
    LegacyPolicyAgreementValidator {
        rules: vec![
            Rc::new(LegacyPolicyAgreementCreatedAtRangeRule { min: 32, max: 3600 })
                as Rc<dyn LegacyPolicyAgreementRule>,
            Rc::new(LegacyPolicyAgreementRequiredCategoryRule {
                message: "legacy policy agreement needs this field".to_string(),
            }),
            Rc::new(LegacyPolicyAgreementCategoryLengthRule { max_length: 80 }),
            Rc::new(LegacyPolicyAgreementTagsLimitRule {
                max_count: 1000,
                banned: vec!["tiered".to_string(), "global".to_string()],
            }),
        ],
    }
}
