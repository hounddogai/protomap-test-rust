use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary usage assignment must pass. Check returns its problems, or none.
pub trait PrimaryUsageAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryUsageAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAssignmentCodePrefixRule {
    pub prefix: String,
}

impl PrimaryUsageAssignmentRule for PrimaryUsageAssignmentCodePrefixRule {
    fn check(&self, item: &PrimaryUsageAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAssignmentRequiredCodeRule {
    pub message: String,
}

impl PrimaryUsageAssignmentRule for PrimaryUsageAssignmentRequiredCodeRule {
    fn check(&self, item: &PrimaryUsageAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAssignmentPrincipalLengthRule {
    pub max_length: i64,
}

impl PrimaryUsageAssignmentRule for PrimaryUsageAssignmentPrincipalLengthRule {
    fn check(&self, item: &PrimaryUsageAssignment) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAssignmentArchivedGuardRule {
    pub limit: f64,
}

impl PrimaryUsageAssignmentRule for PrimaryUsageAssignmentArchivedGuardRule {
    fn check(&self, item: &PrimaryUsageAssignment) -> Vec<String> {
        if item.archived && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while archived", item.rate, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAssignmentRequiredPrincipalRule {
    pub message: String,
}

impl PrimaryUsageAssignmentRule for PrimaryUsageAssignmentRequiredPrincipalRule {
    fn check(&self, item: &PrimaryUsageAssignment) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAssignmentAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl PrimaryUsageAssignmentRule for PrimaryUsageAssignmentAllowedTitleRule {
    fn check(&self, item: &PrimaryUsageAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is expired", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary usage assignment.
#[derive(Debug, Clone)]
pub struct PrimaryUsageAssignmentValidator {
    pub rules: Vec<Rc<dyn PrimaryUsageAssignmentRule>>,
}

impl PrimaryUsageAssignmentValidator {
    pub fn validate(&self, item: &PrimaryUsageAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryUsageAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryUsageAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_usage_assignment_validator() -> PrimaryUsageAssignmentValidator {
    PrimaryUsageAssignmentValidator {
        rules: vec![
            Rc::new(PrimaryUsageAssignmentCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn PrimaryUsageAssignmentRule>,
            Rc::new(PrimaryUsageAssignmentRequiredCodeRule {
                message: "primary usage assignment needs an entry".to_string(),
            }),
            Rc::new(PrimaryUsageAssignmentPrincipalLengthRule { max_length: 86400 }),
            Rc::new(PrimaryUsageAssignmentArchivedGuardRule { limit: 3.5 }),
            Rc::new(PrimaryUsageAssignmentRequiredPrincipalRule {
                message: "primary usage assignment needs an entry".to_string(),
            }),
            Rc::new(PrimaryUsageAssignmentAllowedTitleRule { allowed: Vec::new() }),
        ],
    }
}
