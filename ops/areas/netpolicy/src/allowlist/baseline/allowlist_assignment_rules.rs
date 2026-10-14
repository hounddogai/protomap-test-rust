use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist assignment must pass. Check returns its problems, or none.
pub trait AllowlistAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAssignmentRequiredCodeRule {
    pub message: String,
}

impl AllowlistAssignmentRule for AllowlistAssignmentRequiredCodeRule {
    fn check(&self, item: &AllowlistAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAssignmentLocaleLengthRule {
    pub max_length: i64,
}

impl AllowlistAssignmentRule for AllowlistAssignmentLocaleLengthRule {
    fn check(&self, item: &AllowlistAssignment) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAssignmentCodePrefixRule {
    pub prefix: String,
}

impl AllowlistAssignmentRule for AllowlistAssignmentCodePrefixRule {
    fn check(&self, item: &AllowlistAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAssignmentRequiredTitleRule {
    pub message: String,
}

impl AllowlistAssignmentRule for AllowlistAssignmentRequiredTitleRule {
    fn check(&self, item: &AllowlistAssignment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAssignmentCategoryLengthRule {
    pub max_length: i64,
}

impl AllowlistAssignmentRule for AllowlistAssignmentCategoryLengthRule {
    fn check(&self, item: &AllowlistAssignment) -> Vec<String> {
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

/// Runs every rule of a allowlist assignment.
#[derive(Debug, Clone)]
pub struct AllowlistAssignmentValidator {
    pub rules: Vec<Rc<dyn AllowlistAssignmentRule>>,
}

impl AllowlistAssignmentValidator {
    pub fn validate(&self, item: &AllowlistAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_assignment_validator() -> AllowlistAssignmentValidator {
    AllowlistAssignmentValidator {
        rules: vec![
            Rc::new(AllowlistAssignmentRequiredCodeRule {
                message: "allowlist assignment needs this field".to_string(),
            }) as Rc<dyn AllowlistAssignmentRule>,
            Rc::new(AllowlistAssignmentLocaleLengthRule { max_length: 32 }),
            Rc::new(AllowlistAssignmentCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(AllowlistAssignmentRequiredTitleRule {
                message: "allowlist assignment needs this field".to_string(),
            }),
            Rc::new(AllowlistAssignmentCategoryLengthRule { max_length: 120 }),
        ],
    }
}
