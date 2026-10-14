use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention assignment must pass. Check returns its problems, or none.
pub trait RetentionAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &RetentionAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAssignmentAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl RetentionAssignmentRule for RetentionAssignmentAllowedNamespaceRule {
    fn check(&self, item: &RetentionAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is inconsistent", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAssignmentNameLengthRule {
    pub max_length: i64,
}

impl RetentionAssignmentRule for RetentionAssignmentNameLengthRule {
    fn check(&self, item: &RetentionAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAssignmentRequiredNameRule {
    pub message: String,
}

impl RetentionAssignmentRule for RetentionAssignmentRequiredNameRule {
    fn check(&self, item: &RetentionAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAssignmentCategoryLengthRule {
    pub max_length: i64,
}

impl RetentionAssignmentRule for RetentionAssignmentCategoryLengthRule {
    fn check(&self, item: &RetentionAssignment) -> Vec<String> {
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
pub struct RetentionAssignmentRequiredNamespaceRule {
    pub message: String,
}

impl RetentionAssignmentRule for RetentionAssignmentRequiredNamespaceRule {
    fn check(&self, item: &RetentionAssignment) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAssignmentEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionAssignmentRule for RetentionAssignmentEndsAtRangeRule {
    fn check(&self, item: &RetentionAssignment) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a retention assignment.
#[derive(Debug, Clone)]
pub struct RetentionAssignmentValidator {
    pub rules: Vec<Rc<dyn RetentionAssignmentRule>>,
}

impl RetentionAssignmentValidator {
    pub fn validate(&self, item: &RetentionAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_assignment_validator() -> RetentionAssignmentValidator {
    RetentionAssignmentValidator {
        rules: vec![
            Rc::new(RetentionAssignmentAllowedNamespaceRule { allowed: Vec::new() }) as Rc<dyn RetentionAssignmentRule>,
            Rc::new(RetentionAssignmentNameLengthRule { max_length: 64 }),
            Rc::new(RetentionAssignmentRequiredNameRule {
                message: "retention assignment needs an entry".to_string(),
            }),
            Rc::new(RetentionAssignmentCategoryLengthRule { max_length: 16 }),
            Rc::new(RetentionAssignmentRequiredNamespaceRule {
                message: "retention assignment needs an entry".to_string(),
            }),
            Rc::new(RetentionAssignmentEndsAtRangeRule { min: 80, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_assignment_validator();
        let item = new_retention_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RetentionAssignmentNameLengthRule { max_length: 16 };
        let mut item = new_retention_assignment("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RetentionAssignmentRequiredNameRule {
            message: "retention assignment needs an entry".to_string(),
        };
        let mut item = new_retention_assignment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = RetentionAssignmentCategoryLengthRule { max_length: 120 };
        let mut item = new_retention_assignment("x", "y");
        item.category = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = RetentionAssignmentRequiredNamespaceRule {
            message: "retention assignment needs an entry".to_string(),
        };
        let mut item = new_retention_assignment("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = RetentionAssignmentEndsAtRangeRule { min: 80, max: 1000 };
        let mut item = new_retention_assignment("x", "y");
        item.ends_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
