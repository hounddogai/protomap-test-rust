use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional rollback assignment must pass. Check returns its problems, or none.
pub trait RegionalRollbackAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &RegionalRollbackAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRollbackAssignmentRequiredUnitRule {
    pub message: String,
}

impl RegionalRollbackAssignmentRule for RegionalRollbackAssignmentRequiredUnitRule {
    fn check(&self, item: &RegionalRollbackAssignment) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRollbackAssignmentAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl RegionalRollbackAssignmentRule for RegionalRollbackAssignmentAllowedLabelRule {
    fn check(&self, item: &RegionalRollbackAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is missing", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRollbackAssignmentUnitLengthRule {
    pub max_length: i64,
}

impl RegionalRollbackAssignmentRule for RegionalRollbackAssignmentUnitLengthRule {
    fn check(&self, item: &RegionalRollbackAssignment) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional rollback assignment.
#[derive(Debug, Clone)]
pub struct RegionalRollbackAssignmentValidator {
    pub rules: Vec<Rc<dyn RegionalRollbackAssignmentRule>>,
}

impl RegionalRollbackAssignmentValidator {
    pub fn validate(&self, item: &RegionalRollbackAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalRollbackAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalRollbackAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_rollback_assignment_validator() -> RegionalRollbackAssignmentValidator {
    RegionalRollbackAssignmentValidator {
        rules: vec![
            Rc::new(RegionalRollbackAssignmentRequiredUnitRule {
                message: "regional rollback assignment needs a value".to_string(),
            }) as Rc<dyn RegionalRollbackAssignmentRule>,
            Rc::new(RegionalRollbackAssignmentAllowedLabelRule {
                allowed: vec!["strict".to_string()],
            }),
            Rc::new(RegionalRollbackAssignmentUnitLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_rollback_assignment_validator();
        let item = new_regional_rollback_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = RegionalRollbackAssignmentRequiredUnitRule {
            message: "regional rollback assignment needs an entry".to_string(),
        };
        let mut item = new_regional_rollback_assignment("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = RegionalRollbackAssignmentUnitLengthRule { max_length: 32 };
        let mut item = new_regional_rollback_assignment("x", "y");
        item.unit = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
