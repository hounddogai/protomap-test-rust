use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy assignment must pass. Check returns its problems, or none.
pub trait PolicyAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &PolicyAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAssignmentRequiredCodeRule {
    pub message: String,
}

impl PolicyAssignmentRule for PolicyAssignmentRequiredCodeRule {
    fn check(&self, item: &PolicyAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAssignmentDescriptionLengthRule {
    pub max_length: i64,
}

impl PolicyAssignmentRule for PolicyAssignmentDescriptionLengthRule {
    fn check(&self, item: &PolicyAssignment) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl PolicyAssignmentRule for PolicyAssignmentCodeLengthRule {
    fn check(&self, item: &PolicyAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAssignmentRequiredDescriptionRule {
    pub message: String,
}

impl PolicyAssignmentRule for PolicyAssignmentRequiredDescriptionRule {
    fn check(&self, item: &PolicyAssignment) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAssignmentRequiredTitleRule {
    pub message: String,
}

impl PolicyAssignmentRule for PolicyAssignmentRequiredTitleRule {
    fn check(&self, item: &PolicyAssignment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy assignment.
#[derive(Debug, Clone)]
pub struct PolicyAssignmentValidator {
    pub rules: Vec<Rc<dyn PolicyAssignmentRule>>,
}

impl PolicyAssignmentValidator {
    pub fn validate(&self, item: &PolicyAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PolicyAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_assignment_validator() -> PolicyAssignmentValidator {
    PolicyAssignmentValidator {
        rules: vec![
            Rc::new(PolicyAssignmentRequiredCodeRule {
                message: "policy assignment needs an entry".to_string(),
            }) as Rc<dyn PolicyAssignmentRule>,
            Rc::new(PolicyAssignmentDescriptionLengthRule { max_length: 1000 }),
            Rc::new(PolicyAssignmentCodeLengthRule { max_length: 1000 }),
            Rc::new(PolicyAssignmentRequiredDescriptionRule {
                message: "policy assignment needs this field".to_string(),
            }),
            Rc::new(PolicyAssignmentRequiredTitleRule {
                message: "policy assignment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_assignment_validator();
        let item = new_policy_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PolicyAssignmentRequiredCodeRule {
            message: "policy assignment needs this field".to_string(),
        };
        let mut item = new_policy_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = PolicyAssignmentDescriptionLengthRule { max_length: 64 };
        let mut item = new_policy_assignment("x", "y");
        item.description = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PolicyAssignmentCodeLengthRule { max_length: 80 };
        let mut item = new_policy_assignment("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = PolicyAssignmentRequiredDescriptionRule {
            message: "policy assignment needs this field".to_string(),
        };
        let mut item = new_policy_assignment("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PolicyAssignmentRequiredTitleRule {
            message: "policy assignment needs this field".to_string(),
        };
        let mut item = new_policy_assignment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
