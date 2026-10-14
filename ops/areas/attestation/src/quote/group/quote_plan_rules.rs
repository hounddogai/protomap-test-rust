use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a quote plan must pass. Check returns its problems, or none.
pub trait QuotePlanRule: std::fmt::Debug {
    fn check(&self, item: &QuotePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotePlanNotesLengthRule {
    pub max_length: i64,
}

impl QuotePlanRule for QuotePlanNotesLengthRule {
    fn check(&self, item: &QuotePlan) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotePlanNameLengthRule {
    pub max_length: i64,
}

impl QuotePlanRule for QuotePlanNameLengthRule {
    fn check(&self, item: &QuotePlan) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotePlanAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl QuotePlanRule for QuotePlanAllowedExternalRefRule {
    fn check(&self, item: &QuotePlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is unknown", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotePlanTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl QuotePlanRule for QuotePlanTagsLimitRule {
    fn check(&self, item: &QuotePlan) -> Vec<String> {
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
pub struct QuotePlanExternalRefLengthRule {
    pub max_length: i64,
}

impl QuotePlanRule for QuotePlanExternalRefLengthRule {
    fn check(&self, item: &QuotePlan) -> Vec<String> {
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
pub struct QuotePlanCodePrefixRule {
    pub prefix: String,
}

impl QuotePlanRule for QuotePlanCodePrefixRule {
    fn check(&self, item: &QuotePlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote plan.
#[derive(Debug, Clone)]
pub struct QuotePlanValidator {
    pub rules: Vec<Rc<dyn QuotePlanRule>>,
}

impl QuotePlanValidator {
    pub fn validate(&self, item: &QuotePlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuotePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuotePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_plan_validator() -> QuotePlanValidator {
    QuotePlanValidator {
        rules: vec![
            Rc::new(QuotePlanNotesLengthRule { max_length: 86400 }) as Rc<dyn QuotePlanRule>,
            Rc::new(QuotePlanNameLengthRule { max_length: 64 }),
            Rc::new(QuotePlanAllowedExternalRefRule { allowed: Vec::new() }),
            Rc::new(QuotePlanTagsLimitRule {
                max_count: 64,
                banned: vec!["pending".to_string(), "legacy".to_string()],
            }),
            Rc::new(QuotePlanExternalRefLengthRule { max_length: 86400 }),
            Rc::new(QuotePlanCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
