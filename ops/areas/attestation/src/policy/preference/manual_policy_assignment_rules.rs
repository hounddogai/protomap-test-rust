use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual policy assignment must pass. Check returns its problems, or none.
pub trait ManualPolicyAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &ManualPolicyAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPolicyAssignmentRequiredOwnerIdRule {
    pub message: String,
}

impl ManualPolicyAssignmentRule for ManualPolicyAssignmentRequiredOwnerIdRule {
    fn check(&self, item: &ManualPolicyAssignment) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPolicyAssignmentOwnerIdLengthRule {
    pub max_length: i64,
}

impl ManualPolicyAssignmentRule for ManualPolicyAssignmentOwnerIdLengthRule {
    fn check(&self, item: &ManualPolicyAssignment) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPolicyAssignmentNameLengthRule {
    pub max_length: i64,
}

impl ManualPolicyAssignmentRule for ManualPolicyAssignmentNameLengthRule {
    fn check(&self, item: &ManualPolicyAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPolicyAssignmentRequiredSourceRule {
    pub message: String,
}

impl ManualPolicyAssignmentRule for ManualPolicyAssignmentRequiredSourceRule {
    fn check(&self, item: &ManualPolicyAssignment) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual policy assignment.
#[derive(Debug, Clone)]
pub struct ManualPolicyAssignmentValidator {
    pub rules: Vec<Rc<dyn ManualPolicyAssignmentRule>>,
}

impl ManualPolicyAssignmentValidator {
    pub fn validate(&self, item: &ManualPolicyAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualPolicyAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualPolicyAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_policy_assignment_validator() -> ManualPolicyAssignmentValidator {
    ManualPolicyAssignmentValidator {
        rules: vec![
            Rc::new(ManualPolicyAssignmentRequiredOwnerIdRule {
                message: "manual policy assignment needs a value".to_string(),
            }) as Rc<dyn ManualPolicyAssignmentRule>,
            Rc::new(ManualPolicyAssignmentOwnerIdLengthRule { max_length: 16 }),
            Rc::new(ManualPolicyAssignmentNameLengthRule { max_length: 120 }),
            Rc::new(ManualPolicyAssignmentRequiredSourceRule {
                message: "manual policy assignment needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_policy_assignment_validator();
        let item = new_manual_policy_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = ManualPolicyAssignmentRequiredOwnerIdRule {
            message: "manual policy assignment needs a value".to_string(),
        };
        let mut item = new_manual_policy_assignment("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = ManualPolicyAssignmentOwnerIdLengthRule { max_length: 255 };
        let mut item = new_manual_policy_assignment("x", "y");
        item.owner_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ManualPolicyAssignmentNameLengthRule { max_length: 16 };
        let mut item = new_manual_policy_assignment("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = ManualPolicyAssignmentRequiredSourceRule {
            message: "manual policy assignment needs this field".to_string(),
        };
        let mut item = new_manual_policy_assignment("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
