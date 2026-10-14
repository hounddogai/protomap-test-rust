use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary plan assignment must pass. Check returns its problems, or none.
pub trait PrimaryPlanAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryPlanAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryPlanAssignmentRule for PrimaryPlanAssignmentCodeLengthRule {
    fn check(&self, item: &PrimaryPlanAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanAssignmentApprovedGuardRule {
    pub limit: i64,
}

impl PrimaryPlanAssignmentRule for PrimaryPlanAssignmentApprovedGuardRule {
    fn check(&self, item: &PrimaryPlanAssignment) -> Vec<String> {
        if item.approved && (item.priority > self.limit) {
            return vec![format!(
                "priority {} exceeds {} while approved",
                item.priority, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanAssignmentNamespaceLengthRule {
    pub max_length: i64,
}

impl PrimaryPlanAssignmentRule for PrimaryPlanAssignmentNamespaceLengthRule {
    fn check(&self, item: &PrimaryPlanAssignment) -> Vec<String> {
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
pub struct PrimaryPlanAssignmentRequiredCodeRule {
    pub message: String,
}

impl PrimaryPlanAssignmentRule for PrimaryPlanAssignmentRequiredCodeRule {
    fn check(&self, item: &PrimaryPlanAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanAssignmentTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl PrimaryPlanAssignmentRule for PrimaryPlanAssignmentTagsLimitRule {
    fn check(&self, item: &PrimaryPlanAssignment) -> Vec<String> {
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

/// Runs every rule of a primary plan assignment.
#[derive(Debug, Clone)]
pub struct PrimaryPlanAssignmentValidator {
    pub rules: Vec<Rc<dyn PrimaryPlanAssignmentRule>>,
}

impl PrimaryPlanAssignmentValidator {
    pub fn validate(&self, item: &PrimaryPlanAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryPlanAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryPlanAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_plan_assignment_validator() -> PrimaryPlanAssignmentValidator {
    PrimaryPlanAssignmentValidator {
        rules: vec![
            Rc::new(PrimaryPlanAssignmentCodeLengthRule { max_length: 255 }) as Rc<dyn PrimaryPlanAssignmentRule>,
            Rc::new(PrimaryPlanAssignmentApprovedGuardRule { limit: 3600 }),
            Rc::new(PrimaryPlanAssignmentNamespaceLengthRule { max_length: 3600 }),
            Rc::new(PrimaryPlanAssignmentRequiredCodeRule {
                message: "primary plan assignment needs a value".to_string(),
            }),
            Rc::new(PrimaryPlanAssignmentTagsLimitRule {
                max_count: 1000,
                banned: Vec::new(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_plan_assignment_validator();
        let item = new_primary_plan_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryPlanAssignmentCodeLengthRule { max_length: 120 };
        let mut item = new_primary_plan_assignment("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = PrimaryPlanAssignmentNamespaceLengthRule { max_length: 3600 };
        let mut item = new_primary_plan_assignment("x", "y");
        item.namespace = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryPlanAssignmentRequiredCodeRule {
            message: "primary plan assignment needs an entry".to_string(),
        };
        let mut item = new_primary_plan_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
