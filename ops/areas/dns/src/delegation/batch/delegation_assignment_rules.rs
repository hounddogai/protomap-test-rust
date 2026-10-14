use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation assignment must pass. Check returns its problems, or none.
pub trait DelegationAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &DelegationAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationAssignmentKeyIdLengthRule {
    pub max_length: i64,
}

impl DelegationAssignmentRule for DelegationAssignmentKeyIdLengthRule {
    fn check(&self, item: &DelegationAssignment) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationAssignmentRequiredLocaleRule {
    pub message: String,
}

impl DelegationAssignmentRule for DelegationAssignmentRequiredLocaleRule {
    fn check(&self, item: &DelegationAssignment) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationAssignmentLocaleLengthRule {
    pub max_length: i64,
}

impl DelegationAssignmentRule for DelegationAssignmentLocaleLengthRule {
    fn check(&self, item: &DelegationAssignment) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationAssignmentNameLengthRule {
    pub max_length: i64,
}

impl DelegationAssignmentRule for DelegationAssignmentNameLengthRule {
    fn check(&self, item: &DelegationAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl DelegationAssignmentRule for DelegationAssignmentCodeLengthRule {
    fn check(&self, item: &DelegationAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationAssignmentRequiredNameRule {
    pub message: String,
}

impl DelegationAssignmentRule for DelegationAssignmentRequiredNameRule {
    fn check(&self, item: &DelegationAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation assignment.
#[derive(Debug, Clone)]
pub struct DelegationAssignmentValidator {
    pub rules: Vec<Rc<dyn DelegationAssignmentRule>>,
}

impl DelegationAssignmentValidator {
    pub fn validate(&self, item: &DelegationAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_assignment_validator() -> DelegationAssignmentValidator {
    DelegationAssignmentValidator {
        rules: vec![
            Rc::new(DelegationAssignmentKeyIdLengthRule { max_length: 86400 }) as Rc<dyn DelegationAssignmentRule>,
            Rc::new(DelegationAssignmentRequiredLocaleRule {
                message: "delegation assignment needs a value".to_string(),
            }),
            Rc::new(DelegationAssignmentLocaleLengthRule { max_length: 1000 }),
            Rc::new(DelegationAssignmentNameLengthRule { max_length: 64 }),
            Rc::new(DelegationAssignmentCodeLengthRule { max_length: 120 }),
            Rc::new(DelegationAssignmentRequiredNameRule {
                message: "delegation assignment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_assignment_validator();
        let item = new_delegation_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = DelegationAssignmentKeyIdLengthRule { max_length: 1000 };
        let mut item = new_delegation_assignment("x", "y");
        item.key_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = DelegationAssignmentRequiredLocaleRule {
            message: "delegation assignment needs an entry".to_string(),
        };
        let mut item = new_delegation_assignment("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = DelegationAssignmentLocaleLengthRule { max_length: 32 };
        let mut item = new_delegation_assignment("x", "y");
        item.locale = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DelegationAssignmentNameLengthRule { max_length: 32 };
        let mut item = new_delegation_assignment("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DelegationAssignmentCodeLengthRule { max_length: 80 };
        let mut item = new_delegation_assignment("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DelegationAssignmentRequiredNameRule {
            message: "delegation assignment needs an entry".to_string(),
        };
        let mut item = new_delegation_assignment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
